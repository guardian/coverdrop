package com.theguardian.coverdrop.core.persistence

import androidx.annotation.VisibleForTesting
import com.goterl.lazysodium.SodiumAndroid
import com.theguardian.coverdrop.core.crypto.EncryptionKeyPair
import com.theguardian.coverdrop.core.crypto.PrivateSendingQueueSecret
import com.theguardian.coverdrop.core.models.JournalistId
import com.theguardian.coverdrop.core.utils.LENGTH_ENCODING_OVERHEAD
import com.theguardian.coverdrop.core.utils.deserializeList
import com.theguardian.coverdrop.core.utils.getLengthEncodedByteArray
import com.theguardian.coverdrop.core.utils.putLengthEncodedByteArray
import com.theguardian.coverdrop.core.utils.serializeOrThrow
import java.io.ByteArrayOutputStream
import java.nio.BufferOverflowException
import java.nio.ByteBuffer
import java.time.Instant
import java.util.zip.GZIPInputStream
import java.util.zip.GZIPOutputStream

/**
 * Marker for the currently used serialization code. This is currently important use as there is
 * only one version. However, we will need this forward-compatible flag for later changes.
 */
private const val SERIALIZATION_VERSION_ID = 0x02.toByte()

internal data class MailboxContent(
    val encryptionKeyPair: EncryptionKeyPair,
    val privateSendingQueueSecret: PrivateSendingQueueSecret,
    val messageThreads: List<StoredMessageThread>,
) {
    companion object {
        fun deserialize(bytes: ByteArray): MailboxContent {
            val bootstrapBuffer = ByteBuffer.wrap(bytes)

            // Verify serialization version to enable forward compatibility
            val serializationVersionId = bootstrapBuffer.get()
            check(serializationVersionId == SERIALIZATION_VERSION_ID)

            // Decrypt the rest of the data
            val compressedData = bootstrapBuffer.getLengthEncodedByteArray()
            val uncompressedData = GZIPInputStream(compressedData.inputStream()).use {
                it.readBytes()
            }
            val buffer = ByteBuffer.wrap(uncompressedData)

            return MailboxContent(
                encryptionKeyPair = EncryptionKeyPair.deserialize(buffer.getLengthEncodedByteArray()),
                privateSendingQueueSecret = PrivateSendingQueueSecret.deserialize(buffer.getLengthEncodedByteArray()),
                messageThreads = deserializeList(
                    bytes = buffer.getLengthEncodedByteArray(),
                    deserializeElement = { StoredMessageThread.deserialize(it) }
                ),
            )
        }

        fun newEmptyMailbox(libSodium: SodiumAndroid): MailboxContent {
            val encryptionKeyPair = EncryptionKeyPair.new(libSodium)
            val privateSendingQueueSecret = PrivateSendingQueueSecret.fromSecureRandom()

            return MailboxContent(
                encryptionKeyPair = encryptionKeyPair,
                privateSendingQueueSecret = privateSendingQueueSecret,
                messageThreads = emptyList(),
            )
        }
    }

    /**
     * [paddedOutputSize] provides the size of the returned [ByteArray]. If the serialization would
     * exceed this value, the oldest messages are deleted based on [Message.timestamp].
     *
     * If the data still does not fit within the given [paddedOutputSize] a
     * [java.nio.BufferOverflowException] is thrown which indicates that the value is incorrect.
     */
    fun serializeOrTruncate(paddedOutputSize: Int): ByteArray {
        val outerBuffer = ByteBuffer.allocate(paddedOutputSize)

        outerBuffer.put(SERIALIZATION_VERSION_ID)

        val bufferPositionBeforeMainPayload = outerBuffer.position()
        val remainingBytes = outerBuffer.remaining() - LENGTH_ENCODING_OVERHEAD
        check(remainingBytes > 0)

        var currentMessageThreads = messageThreads
        while (true) {
            try {
                val mainPayload =
                    serializeMainPayloadOrThrow(paddedOutputSize, currentMessageThreads)

                // This will throw if the compressed data is still too large
                outerBuffer.putLengthEncodedByteArray(mainPayload)

                break
            } catch (_: BufferOverflowException) {
                // try again without the oldest message; while this seems expensive, we generally
                // don't expect that more than 1-2 messages have been added since we last
                // successfully serialized and deserialized the mailbox
                currentMessageThreads = currentMessageThreads.copyWithoutOldestMessage()

                // importantly: restore the buffer position to the state before the main payload
                outerBuffer.position(bufferPositionBeforeMainPayload)
            }
        }

        // return the full-length buffer to match the final padded length
        return outerBuffer.array()
    }

    private fun serializeMainPayloadOrThrow(
        paddedOutputSize: Int,
        currentMessageThreads: List<StoredMessageThread>,
    ): ByteArray {
        // The inner buffer will be compressed in the next steps; therefore we allocate it a larger
        // size and hope the compression gets us below the limit; if that fails, we will throw
        // a BufferOverflowException and the caller will try again with fewer messages.
        val innerBuffer = ByteBuffer.allocate(4 * paddedOutputSize)

        innerBuffer.putLengthEncodedByteArray(encryptionKeyPair.serialize())
        innerBuffer.putLengthEncodedByteArray(privateSendingQueueSecret.serialize())
        innerBuffer.putLengthEncodedByteArray(
            currentMessageThreads.serializeOrThrow(
                maxSize = innerBuffer.remaining(),
                serializeElement = { it.serialize(maxSize = paddedOutputSize) })
        )

        // Compress the inner buffer
        val compressedData = ByteArrayOutputStream().use { outputStream ->
            GZIPOutputStream(outputStream).use { gzipStream ->
                gzipStream.write(innerBuffer.array())
            }
            outputStream.toByteArray()
        }
        return compressedData
    }

    /**
     * [paddedOutputSize] provides the size of the returned [ByteArray]. If the serialization would
     * exceed this value a [java.nio.BufferOverflowException] is thrown.
     *
     * Only used for testing.
     */
    @VisibleForTesting
    @kotlin.jvm.Throws(BufferOverflowException::class)
    fun serializeOrThrow(paddedOutputSize: Int): ByteArray {
        val outerBuffer = ByteBuffer.allocate(paddedOutputSize)
        outerBuffer.put(SERIALIZATION_VERSION_ID)

        val mainPayload = serializeMainPayloadOrThrow(paddedOutputSize, messageThreads)

        // This will throw if the compressed data is still too large
        outerBuffer.putLengthEncodedByteArray(mainPayload)

        // return the full-length buffer to match the final padded length
        return outerBuffer.array()
    }

    /**
     * Given a [JournalistId], find the corresponding [StoredMessageThread]
     */
    internal fun getThreadWithId(id: JournalistId): StoredMessageThread? {
        return this.messageThreads.find { it.recipientId == id }
    }

    /**
     * Returns the message threads with in the [mailbox] as a list of [StoredMessageThread]s
     */
    internal fun getMessageThreads(): List<StoredMessageThread> {
        return this.messageThreads
    }

    /**
     * Makes a copy of the [MailboxContent] but with a [StoredMessageThread] replaced
     */
    internal fun copyWithNewThread(new: StoredMessageThread): MailboxContent {
        val existingThreads = this.messageThreads.toMutableList()
        existingThreads.removeAll { it.recipientId == new.recipientId }
        existingThreads.add(new)
        return this.copy(messageThreads = existingThreads)
    }

    /**
     * Makes a copy of the [MailboxContent] but removes all messages older than the given [cutoff].
     */
    fun copyMinusOldMessages(cutoff: Instant): MailboxContent {
        val newThreads = messageThreads.map { it.copyAndRemoveOlderMessages(cutoff) }
        return this.copy(messageThreads = newThreads)
    }
}

