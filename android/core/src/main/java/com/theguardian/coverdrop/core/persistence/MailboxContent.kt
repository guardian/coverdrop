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
import java.nio.BufferOverflowException
import java.nio.ByteBuffer
import java.time.Instant
import java.util.zip.GZIPInputStream

/**
 * Current serialization version: the fields are written directly without compression.
 */
private const val SERIALIZATION_VERSION_V3 = 0x03.toByte()

/**
 * Legacy serialization version where the payload was wrapped in a length-encoded GZIP blob. It is
 * only supported for reading; the next save will rewrite the mailbox using the current version.
 */
private const val SERIALIZATION_VERSION_V2 = 0x02.toByte()

internal data class MailboxContent(
    val encryptionKeyPair: EncryptionKeyPair,
    val privateSendingQueueSecret: PrivateSendingQueueSecret,
    val messageThreads: List<StoredMessageThread>,
) {
    companion object {
        fun deserialize(bytes: ByteArray): MailboxContent {
            val buffer = ByteBuffer.wrap(bytes)

            return when (val serializationVersionId = buffer.get()) {
                SERIALIZATION_VERSION_V3 -> deserializePayload(buffer)
                SERIALIZATION_VERSION_V2 -> deserializeLegacyV2(buffer)
                else -> throw IllegalStateException(
                    "Unknown serialization version: $serializationVersionId"
                )
            }
        }

        private fun deserializePayload(buffer: ByteBuffer) = MailboxContent(
            encryptionKeyPair = EncryptionKeyPair.deserialize(buffer.getLengthEncodedByteArray()),
            privateSendingQueueSecret = PrivateSendingQueueSecret.deserialize(buffer.getLengthEncodedByteArray()),
            messageThreads = deserializeList(
                bytes = buffer.getLengthEncodedByteArray(),
                deserializeElement = { StoredMessageThread.deserialize(it) }
            ),
        )

        /**
         * The legacy V2 format wrapped the payload in a length-encoded GZIP blob; the uncompressed
         * payload is identical to the current format.
         */
        private fun deserializeLegacyV2(buffer: ByteBuffer): MailboxContent {
            val compressedData = buffer.getLengthEncodedByteArray()
            val uncompressedData = GZIPInputStream(compressedData.inputStream()).use {
                it.readBytes()
            }
            return deserializePayload(ByteBuffer.wrap(uncompressedData))
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
        var currentMessageThreads = messageThreads
        while (true) {
            try {
                return serializeOrThrow(paddedOutputSize, currentMessageThreads)
            } catch (_: BufferOverflowException) {
                // try again without the oldest message; while this seems expensive, we generally
                // don't expect that more than 1-2 messages have been added since we last
                // successfully serialized and deserialized the mailbox
                currentMessageThreads = currentMessageThreads.copyWithoutOldestMessage()
            }
        }
    }

    /**
     * [paddedOutputSize] provides the size of the returned [ByteArray]. If the serialization would
     * exceed this value a [java.nio.BufferOverflowException] is thrown.
     *
     * Only used for testing.
     */
    @VisibleForTesting
    @kotlin.jvm.Throws(BufferOverflowException::class)
    fun serializeOrThrow(paddedOutputSize: Int): ByteArray =
        serializeOrThrow(paddedOutputSize, messageThreads)

    private fun serializeOrThrow(
        paddedOutputSize: Int,
        currentMessageThreads: List<StoredMessageThread>,
    ): ByteArray {
        val buffer = ByteBuffer.allocate(paddedOutputSize)

        buffer.put(SERIALIZATION_VERSION_V3)
        buffer.putLengthEncodedByteArray(encryptionKeyPair.serialize())
        buffer.putLengthEncodedByteArray(privateSendingQueueSecret.serialize())
        buffer.putLengthEncodedByteArray(
            currentMessageThreads.serializeOrThrow(
                maxSize = buffer.remaining() - LENGTH_ENCODING_OVERHEAD,
                serializeElement = { it.serialize(maxSize = paddedOutputSize) })
        )

        // return the full-length buffer to match the final padded length; the trailing zero
        // padding is ignored during deserialization because every field is length-encoded
        return buffer.array()
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

