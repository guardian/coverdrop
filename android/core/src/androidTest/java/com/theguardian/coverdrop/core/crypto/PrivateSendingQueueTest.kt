package com.theguardian.coverdrop.core.crypto

import com.google.common.truth.Truth.assertThat
import com.theguardian.coverdrop.core.utils.nextByteArray
import com.theguardian.coverdrop.core.utils.padTo
import kotlinx.coroutines.runBlocking
import org.junit.Assert.fail
import org.junit.Test
import java.security.SecureRandom


class PrivateSendingQueueTest {
    private val secret = PrivateSendingQueueSecret("secret__secret__".toByteArray())
    private val message1 = PrivateSendingQueueItem(
        bytes = "message 1".toByteArray().padTo(COVERDROP_PRIVATE_SENDING_QUEUE_ITEM_SIZE)
    )
    private val message2 = PrivateSendingQueueItem(
        bytes = "message 2".toByteArray().padTo(COVERDROP_PRIVATE_SENDING_QUEUE_ITEM_SIZE)
    )

    private fun createCoverItem() = PrivateSendingQueueItem(
        bytes = SecureRandom().nextByteArray(COVERDROP_PRIVATE_SENDING_QUEUE_ITEM_SIZE)
    )

    @Test
    fun testEnqueue_whenAddingMessage_thenFillLevelIncreases(): Unit = runBlocking {
        val queue = CoverDropPrivateSendingQueue.empty(::createCoverItem)
        assertThat(queue.getRealMessageCount(secret)).isEqualTo(0)

        queue.enqueue(secret, message1)
        assertThat(queue.getRealMessageCount(secret)).isEqualTo(1)

        queue.enqueue(secret, message2)
        assertThat(queue.getRealMessageCount(secret)).isEqualTo(2)
    }

    @Test
    fun testEnqueue_whenCheckingFillLevelWithWrongSecret_thenReturnsEmpty(): Unit = runBlocking {
        val wrongSecret = PrivateSendingQueueSecret("__terces__terces".toByteArray())

        val queue = CoverDropPrivateSendingQueue.empty(::createCoverItem)
        assertThat(queue.getRealMessageCount(secret)).isEqualTo(0)

        queue.enqueue(secret, message1)
        assertThat(queue.getRealMessageCount(wrongSecret)).isEqualTo(0)

        queue.enqueue(secret, message2)
        assertThat(queue.getRealMessageCount(wrongSecret)).isEqualTo(0)
    }

    @Test
    fun testEnqueue_whenAddingMessagesBeyondCapacity_thanSpaceThenThrows(): Unit = runBlocking {
        val queue = CoverDropPrivateSendingQueue.empty(::createCoverItem)
        for (i in 0 until COVERDROP_PRIVATE_SENDING_QUEUE_N) {
            queue.enqueue(secret, message1)
        }

        // the next message overflows the queue
        try {
            queue.enqueue(secret, message1)
            fail("no exception thrown")
        } catch (e: Exception) {
            assertThat(e::class).isEqualTo(IllegalStateException::class)
        }
    }

    @Test
    fun testEnqueue_whenAddingMessageWithDifferentSecret_thenOthersOverwritten(): Unit =
        runBlocking {
            val differentSecret = PrivateSendingQueueSecret("__terces__terces".toByteArray())
            val message3 = PrivateSendingQueueItem(
                bytes = "message 3".toByteArray().padTo(COVERDROP_PRIVATE_SENDING_QUEUE_ITEM_SIZE)
            )

            val queue = CoverDropPrivateSendingQueue.empty(::createCoverItem)
            queue.enqueue(secret, message1)
            queue.enqueue(secret, message2)
            queue.enqueue(differentSecret, message3)

            // we would have otherwise expected message1 due to the FIFO nature of the queue
            assertThat(queue.dequeue(::createCoverItem)).isEqualTo(message3)
        }

    @Test
    fun testDequeue_whenAddedMessages_thenPoppedInOrder(): Unit = runBlocking {
        val queue = CoverDropPrivateSendingQueue.empty(::createCoverItem)
        queue.enqueue(secret, message1)
        queue.enqueue(secret, message2)

        assertThat(queue.dequeue(::createCoverItem)).isEqualTo(message1)
        assertThat(queue.dequeue(::createCoverItem)).isEqualTo(message2)
    }

    @Test
    fun testDequeue_whenPoppingMoreThanRealMessages_thenCoverMessagesReturned(): Unit =
        runBlocking {
            val queue = CoverDropPrivateSendingQueue.empty(::createCoverItem)
            queue.enqueue(secret, message1)
            queue.enqueue(secret, message2)

            assertThat(queue.dequeue(::createCoverItem)).isEqualTo(message1)
            assertThat(queue.dequeue(::createCoverItem)).isEqualTo(message2)

            val cover = queue.dequeue(::createCoverItem)
            assertThat(cover).isNotEqualTo(message1)
            assertThat(cover).isNotEqualTo(message2)
        }

    @Test
    fun testClear_whenCleared_thenContainsOnlyCoverMessages(): Unit = runBlocking {
        val queue = CoverDropPrivateSendingQueue.empty(::createCoverItem)
        queue.enqueue(secret, message1)
        queue.enqueue(secret, message2)

        assertThat(queue.getRealMessageCount(secret)).isEqualTo(2)

        queue.clear(secret, ::createCoverItem)

        assertThat(queue.getRealMessageCount(secret)).isEqualTo(0)
    }

    @Test
    fun testFromBytes_whenSerdeEmpty_thenDeserializesSuccessfully(): Unit = runBlocking {
        val original = CoverDropPrivateSendingQueue.empty(::createCoverItem)
        CoverDropPrivateSendingQueue.fromBytes(original.serialize())
    }

    @Test
    fun testFromBytes_whenSerdeWithMessages_thenDeserializesSuccessfully(): Unit = runBlocking {
        val original = CoverDropPrivateSendingQueue.empty(::createCoverItem)
        original.enqueue(secret, message1)
        original.enqueue(secret, message2)

        val copy = CoverDropPrivateSendingQueue.fromBytes(
            original.serialize(),
        )

        val fillLevel = copy.getRealMessageCount(secret)
        assertThat(fillLevel).isEqualTo(2)

        val actualMessage1 = copy.dequeue(::createCoverItem)
        assertThat(actualMessage1).isEqualTo(message1)

        val actualMessage2 = copy.dequeue(::createCoverItem)
        assertThat(actualMessage2).isEqualTo(message2)

        original.dequeue(::createCoverItem) // message 1
        original.dequeue(::createCoverItem) // message 2
        val originalCover1 = original.dequeue(::createCoverItem)
        val actualCover1 = copy.dequeue(::createCoverItem)
        assertThat(originalCover1).isEqualTo(actualCover1)
    }
}
