package com.theguardian.coverdrop.core.persistence

import com.google.common.truth.Truth.assertThat
import com.theguardian.coverdrop.core.crypto.PrivateSendingQueueHint
import com.theguardian.coverdrop.core.encryptedstorage.CONTENT_BLOB_LEN_BYTES
import com.theguardian.coverdrop.core.utils.StoredMessageSubject
import com.theguardian.coverdrop.core.utils.StoredMessageThreadsSubject
import com.theguardian.coverdrop.core.utils.hexEncode
import org.junit.Test
import java.time.Duration
import java.time.Instant
import kotlin.random.Random


class StoredMessageThreadsTest {

    private val maxSize = CONTENT_BLOB_LEN_BYTES

    @Test
    fun testStoredMessageThreadTotalMessageCount_whenEmpty_thenReturns0() {
        val x: StoredMessageThreads = emptyList()
        assertThat(x.totalMessageCount()).isEqualTo(0)
    }

    @Test
    fun testStoredMessageThreadTotalMessageCount_whenMultipleThreads_thenReturnsSum() {
        val random = Random(seed = 0)

        val x = listOf(
            StoredMessageThread("j1", List(3) { randomStoredMessage(random) }),
            StoredMessageThread("j2", List(0) { randomStoredMessage(random) }),
            StoredMessageThread("j3", List(1) { randomStoredMessage(random) })
        )
        assertThat(x.totalMessageCount()).isEqualTo(4)
    }

    @Test
    fun testStoredMessagesThreadRemoveOldest_whenFuzzed_thenMatchesInvariant() {
        val random = Random(seed = 0)

        for (i in 0 until 10) {
            val threads = List(random.nextInt(5, 15)) { randomStoredMessageThread(random) }

            val after = threads.copyWithoutOldestMessage()
            assertThat(after.totalMessageCount()).isEqualTo(threads.totalMessageCount() - 1)
        }
    }

    @Test
    fun testStoredMessagesThreadRemoveOldest_whenGivenSomeThreads_thenOldestIndeedRemovedandEmtpyGarbageCollected() {
        val t1 = StoredMessageThread(
            recipientId = "j1",
            messages = listOf(
                StoredMessage.localForTest(Instant.ofEpochMilli(100), "j1a"),
                StoredMessage.localForTest(Instant.ofEpochMilli(200), "j1b"),
            )
        )
        val t2 = StoredMessageThread(
            recipientId = "j2",
            messages = listOf(
                StoredMessage.localForTest(Instant.ofEpochMilli(80), "j2a"),
            )
        )
        val t3 = StoredMessageThread(
            recipientId = "j3",
            messages = listOf(
                StoredMessage.localForTest(Instant.ofEpochMilli(50), "j3a"),
                StoredMessage.localForTest(Instant.ofEpochMilli(400), "j3b"),
            )
        )

        val threads1 = listOf(t1, t2, t3)
        assertThat(threads1.size).isEqualTo(3)
        assertThat(threads1.totalMessageCount()).isEqualTo(5)

        //
        // First remove operation should remove the message "j3a".
        //

        val threads2 = threads1.copyWithoutOldestMessage()
        assertThat(threads2.size).isEqualTo(3)
        assertThat(threads2.totalMessageCount()).isEqualTo(4)

        val t3new = StoredMessageThread(
            recipientId = "j3",
            messages = listOf(
                StoredMessage.localForTest(Instant.ofEpochMilli(400), "j3b"),
            )
        )

        assertThat(threads2).isEqualTo(listOf(t1, t2, t3new))

        //
        // Second remove operation should remove the message "j2a" AND then garbage collect the
        // then empty thread t2.
        //

        val threads3 = threads2.copyWithoutOldestMessage()
        assertThat(threads3.size).isEqualTo(2)
        assertThat(threads3.totalMessageCount()).isEqualTo(3)
        assertThat(threads3).isEqualTo(listOf(t1, t3new)) // without t2
    }

    @Test
    fun testSerializeDeserializeStoredMessage_whenEmpty_thenResultEmpty() {
        val original = StoredMessage.localForTest(timestamp = Instant.now(), message = "")

        val serialized = original.serialize()

        val actual = StoredMessage.deserialize(serialized)
        StoredMessageSubject.assertThat(actual).isEqualTo(original)
        assertThat(actual.payload).isEmpty()
    }

    @Test
    fun testSerializeDeserializeStoredMessage_whenNonEmpty_thenMatches() {
        val original = StoredMessage.local(
            timestamp = Instant.now(),
            message = "hello",
            privateSendingQueueHint = PrivateSendingQueueHint("0123456789ABCDEF".encodeToByteArray())
        )

        val serialized = original.serialize()

        val actual = StoredMessage.deserialize(serialized)
        StoredMessageSubject.assertThat(actual).isEqualTo(original)
    }

    @Test
    fun testSerializeDeserializeStoredMessageThread_whenEmpty_thenResultEmpty() {
        val original = StoredMessageThread(recipientId = "", messages = emptyList())

        val serialized = original.serialize(maxSize)

        val actual = StoredMessageThread.deserialize(serialized)
        assertThat(actual).isEqualTo(original)
        assertThat(actual.recipientId).isEmpty()
        assertThat(actual.messages).isEmpty()
    }

    @Test
    fun testSerializeDeserializeStoredMessageThread_whenNonEmpty_thenMatches() {
        val original = StoredMessageThread(
            recipientId = "acbdef",
            messages = listOf(
                StoredMessage.localForTest(timestamp = Instant.now(), message = "ping"),
                StoredMessage.remote(timestamp = Instant.now(), message = "pong"),
            )
        )

        val serialized = original.serialize(maxSize)

        val actual = StoredMessageThread.deserialize(serialized)
        StoredMessageThreadsSubject.assertThat(actual).isEqualTo(original)
    }
}

internal fun randomStoredMessageThread(random: Random): StoredMessageThread {
    return StoredMessageThread(
        recipientId = random.nextBytes(10).hexEncode(),
        messages = List(random.nextInt(5, 10)) { randomStoredMessage(random) }
    )
}

internal fun randomStoredMessage(random: Random): StoredMessage {
    val timestamp = Instant.parse("2023-12-24T12:00:00Z") +
            Duration.ofSeconds(random.nextLong(until = 30 * 24 * 3600))

    val message = random.nextBytes(random.nextInt(10, 100)).decodeToString()
    return if (random.nextBoolean()) {
        StoredMessage.localForTest(timestamp = timestamp, message = message)
    } else {
        StoredMessage.remote(timestamp = timestamp, message = message)
    }
}
