package com.theguardian.coverdrop.core.persistence

import com.google.common.truth.Truth.assertThat
import com.theguardian.coverdrop.core.createLibSodium
import com.theguardian.coverdrop.core.crypto.EncryptionKeyPair
import com.theguardian.coverdrop.core.crypto.PrivateSendingQueueSecret
import com.theguardian.coverdrop.core.utils.putLengthEncodedByteArray
import com.theguardian.coverdrop.core.utils.serializeOrThrow
import org.junit.Test
import java.io.ByteArrayOutputStream
import java.nio.BufferOverflowException
import java.nio.ByteBuffer
import java.time.Duration
import java.time.Instant
import java.util.zip.GZIPOutputStream
import kotlin.random.Random


class MailboxContentTest {

    private val libSodium = createLibSodium()

    // choosing a smaller size than CONTENT_BLOB_LEN_BYTES to test truncation
    private val maxSize = 32 * 1024

    @Test
    fun testSerializeDeserializeMailBox_whenEmptyMailbox_thenResultMatches() {
        val original = MailboxContent.newEmptyMailbox(libSodium)

        val serialized = original.serializeOrThrow(maxSize)
        assertThat(serialized.size).isEqualTo(maxSize)

        val actual = MailboxContent.deserialize(serialized)
        assertThat(actual).isEqualTo(original)
    }

    @Test
    fun testSerializeDeserializeMailBox_whenMailboxWithFuzzedContent_thenResultMatches() {
        val random = Random(seed = 0)
        val messageThreads = List(3) { randomStoredMessageThread(random) }

        val original = MailboxContent(
            encryptionKeyPair = EncryptionKeyPair.new(libSodium),
            privateSendingQueueSecret = PrivateSendingQueueSecret.fromSecureRandom(),
            messageThreads = messageThreads,
        )

        val serialized = original.serializeOrThrow(maxSize)
        assertThat(serialized.size).isEqualTo(maxSize)

        val actual = MailboxContent.deserialize(serialized)
        assertThat(original).isEqualTo(actual)
    }

    @Test(expected = BufferOverflowException::class)
    fun testSerializeDeserializeMailBoxWithThrow_whenTooMuchContent_thenThrows() {
        val random = Random(seed = 0)
        val messageThreads = List(100) { randomStoredMessageThread(random) }

        val original = MailboxContent(
            encryptionKeyPair = EncryptionKeyPair.new(libSodium),
            privateSendingQueueSecret = PrivateSendingQueueSecret.fromSecureRandom(),
            messageThreads = messageThreads,
        )

        original.serializeOrThrow(maxSize)
    }

    @Test
    fun testSerializeDeserializeMailBoxWithTruncate_whenTooMuchContent_thenTruncates() {
        val random = Random(seed = 0)
        val messageThreads = List(100) { randomStoredMessageThread(random) }

        val original = MailboxContent(
            encryptionKeyPair = EncryptionKeyPair.new(libSodium),
            privateSendingQueueSecret = PrivateSendingQueueSecret.fromSecureRandom(),
            messageThreads = messageThreads,
        )

        val serialized = original.serializeOrTruncate(maxSize)
        assertThat(serialized.size).isEqualTo(maxSize)

        val actual = MailboxContent.deserialize(serialized)

        // everything should be stored as usual (except for loosing some old messages)
        assertThat(actual.encryptionKeyPair).isEqualTo(original.encryptionKeyPair)
        assertThat(actual.privateSendingQueueSecret).isEqualTo(original.privateSendingQueueSecret)
        assertThat(actual.messageThreads.totalMessageCount())
            .isLessThan(original.messageThreads.totalMessageCount())
    }

    @Test
    fun testDeserialize_whenGivenLegacyV2Format_thenContentMatchesAndReserializesAsV3() {
        val random = Random(seed = 0)

        val original = MailboxContent(
            encryptionKeyPair = EncryptionKeyPair.new(libSodium),
            privateSendingQueueSecret = PrivateSendingQueueSecret.fromSecureRandom(),
            messageThreads = List(3) { randomStoredMessageThread(random) },
        )

        // reading the legacy V2 format yields the original content
        val legacySerialized = serializeLegacyV2Format(original, maxSize)
        val actual = MailboxContent.deserialize(legacySerialized)
        assertThat(actual).isEqualTo(original)

        // re-serializing (as happens on unlock) writes the V3 format which round-trips
        val reserialized = actual.serializeOrThrow(maxSize)
        assertThat(reserialized[0]).isEqualTo(0x03.toByte())
        assertThat(MailboxContent.deserialize(reserialized)).isEqualTo(original)
    }

    @Test(expected = IllegalStateException::class)
    fun testDeserialize_whenUnknownVersion_thenThrows() {
        val serialized = MailboxContent.newEmptyMailbox(libSodium).serializeOrThrow(maxSize)
        serialized[0] = 0x7F
        MailboxContent.deserialize(serialized)
    }

    /**
     * Replicates the legacy V2 serialization (a length-encoded GZIP payload) that has since been
     * removed from [MailboxContent]; copied verbatim from the old implementation.
     */
    private fun serializeLegacyV2Format(
        content: MailboxContent,
        paddedOutputSize: Int,
    ): ByteArray {
        val innerBuffer = ByteBuffer.allocate(4 * paddedOutputSize)
        innerBuffer.putLengthEncodedByteArray(content.encryptionKeyPair.serialize())
        innerBuffer.putLengthEncodedByteArray(content.privateSendingQueueSecret.serialize())
        innerBuffer.putLengthEncodedByteArray(
            content.messageThreads.serializeOrThrow(
                maxSize = innerBuffer.remaining(),
                serializeElement = { it.serialize(maxSize = paddedOutputSize) })
        )

        val compressedData = ByteArrayOutputStream().use { outputStream ->
            GZIPOutputStream(outputStream).use { gzipStream ->
                gzipStream.write(innerBuffer.array())
            }
            outputStream.toByteArray()
        }

        val outerBuffer = ByteBuffer.allocate(paddedOutputSize)
        outerBuffer.put(0x02.toByte())
        outerBuffer.putLengthEncodedByteArray(compressedData)
        return outerBuffer.array()
    }

    @Test
    fun testRemoveOldMessages_whenNoneOlderThanCutoff_thenNothingTruncated() {
        val random = Random(seed = 0)
        val messageThreads = List(3) { randomStoredMessageThread(random) }

        val original = MailboxContent(
            encryptionKeyPair = EncryptionKeyPair.new(libSodium),
            privateSendingQueueSecret = PrivateSendingQueueSecret.fromSecureRandom(),
            messageThreads = messageThreads,
        )

        val cutoff = original.messageThreads.minOf { it.messages.minOf { m -> m.timestamp } }
        val actual = original.copyMinusOldMessages(cutoff)

        assertThat(actual).isEqualTo(original)
    }

    @Test
    fun testRemoveOldMessages_whenAllOlderThanCutoff_thenAllThreadsEmpty() {
        val random = Random(seed = 0)
        val messageThreads = List(3) { randomStoredMessageThread(random) }

        val original = MailboxContent(
            encryptionKeyPair = EncryptionKeyPair.new(libSodium),
            privateSendingQueueSecret = PrivateSendingQueueSecret.fromSecureRandom(),
            messageThreads = messageThreads,
        )

        val newest = original.messageThreads.maxOf { it.messages.maxOf { m -> m.timestamp } }
        val cutoff = newest + Duration.ofMillis(1)
        val actual = original.copyMinusOldMessages(cutoff)

        assertThat(actual.messageThreads).hasSize(original.messageThreads.size)
        actual.messageThreads.forEach { thread -> assertThat(thread.messages).isEmpty() }
    }

    @Test
    fun testRemoveOldMessages_whenCuttingOfTheOldestMessage_thenOnlyThoseRemoved() {
        val messageThreads = listOf(
            StoredMessageThread(
                recipientId = "1",
                messages = listOf(
                    StoredMessage.localForTest(timestamp = Instant.ofEpochMilli(10), message = "1"),
                    StoredMessage.localForTest(timestamp = Instant.ofEpochMilli(20), message = "2"),
                    StoredMessage.localForTest(timestamp = Instant.ofEpochMilli(30), message = "3"),
                )
            ),
            StoredMessageThread(
                recipientId = "2",
                messages = listOf(
                    StoredMessage.localForTest(timestamp = Instant.ofEpochMilli(40), message = "4"),
                    StoredMessage.localForTest(timestamp = Instant.ofEpochMilli(50), message = "5"),
                    StoredMessage.localForTest(timestamp = Instant.ofEpochMilli(60), message = "6"),
                )
            ),
        )

        val original = MailboxContent(
            encryptionKeyPair = EncryptionKeyPair.new(libSodium),
            privateSendingQueueSecret = PrivateSendingQueueSecret.fromSecureRandom(),
            messageThreads = messageThreads,
        )

        // first cut removes two messages from one thread
        val afterFirstCut = original.copyMinusOldMessages(Instant.ofEpochMilli(25))
        val expectedThreadsAfterFirstCut = listOf(
            StoredMessageThread(
                recipientId = "1",
                messages = listOf(
                    StoredMessage.localForTest(timestamp = Instant.ofEpochMilli(30), message = "3"),
                )
            ),
            StoredMessageThread(
                recipientId = "2",
                messages = listOf(
                    StoredMessage.localForTest(timestamp = Instant.ofEpochMilli(40), message = "4"),
                    StoredMessage.localForTest(timestamp = Instant.ofEpochMilli(50), message = "5"),
                    StoredMessage.localForTest(timestamp = Instant.ofEpochMilli(60), message = "6"),
                )
            ),
        )
        assertThat(afterFirstCut.messageThreads).isEqualTo(expectedThreadsAfterFirstCut)

        // second cut leaves the first thread empty, and removes one message from the other thread
        val afterSecondCut = afterFirstCut.copyMinusOldMessages(Instant.ofEpochMilli(45))
        val expectedThreadsAfterSecondCut = listOf(
            StoredMessageThread(
                recipientId = "1",
                messages = emptyList()
            ),
            StoredMessageThread(
                recipientId = "2",
                messages = listOf(
                    StoredMessage.localForTest(timestamp = Instant.ofEpochMilli(50), message = "5"),
                    StoredMessage.localForTest(timestamp = Instant.ofEpochMilli(60), message = "6"),
                )
            ),
        )
        assertThat(afterSecondCut.messageThreads).isEqualTo(expectedThreadsAfterSecondCut)
    }
}
