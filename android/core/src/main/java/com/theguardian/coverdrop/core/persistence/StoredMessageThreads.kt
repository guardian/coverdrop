package com.theguardian.coverdrop.core.persistence

import com.theguardian.coverdrop.core.models.JournalistId
import com.theguardian.coverdrop.core.utils.*
import java.nio.BufferOverflowException
import java.nio.ByteBuffer
import java.time.Instant

internal typealias StoredMessageThreads = List<StoredMessageThread>

/**
 * Returns the total number of messages in all threads.
 */
internal fun StoredMessageThreads.totalMessageCount(): Int = sumOf { it.messages.size }

/**
 * Creates a new [StoredMessageThreads] without the oldest message. The result will have a
 * [totalMessageCount] that is exactly 1 less after than before execution. Hence, it requires that
 * there is at least one message in at least one thread.
 *
 * If a thread ends up without any message in it, it is "garbage collected" and removed entirely.
 *
 * Implementation note: where possible we reuse the [StoredMessageThread] objects where no message
 * is removed from.
 */
internal fun StoredMessageThreads.copyWithoutOldestMessage(): StoredMessageThreads {
    require(totalMessageCount() > 0)

    // find oldest message
    var oldestMessage: StoredMessage? = null
    for (thread in this) {
        for (message in thread.messages) {
            if (oldestMessage == null || message.timestamp < oldestMessage.timestamp) {
                oldestMessage = message
            }
        }
    }
    check(oldestMessage != null) // must be true as we checked `totalMessageCount() > 0`

    // create copy where we replace the StoredMessageThread which contains the oldest message
    val copy = this
        .map { thread ->
            if (thread.messages.contains(oldestMessage)) {
                // this is the thread with the oldest message -> filter that message out
                StoredMessageThread(
                    recipientId = thread.recipientId,
                    messages = thread.messages.filterNot { m -> m == oldestMessage }
                )
            } else {
                // "this is not the thread you are looking for"
                thread
            }
        }
        .filterNot { it.messages.isEmpty() } // remove empty threads
        .toList()

    // invariant as per documentation
    check(copy.totalMessageCount() == totalMessageCount() - 1)
    return copy
}

/**
 * Makes a copy of the [StoredMessageThread] but with a new message appended to the end of the messages
 */
internal fun StoredMessageThread.copyWithNewMessage(new: StoredMessage): StoredMessageThread {
    val existingMessages = this.messages.toMutableList()
    existingMessages.add(new)

    return this.copy(messages = existingMessages)
}

/**
 * A message thread as stored within [MailboxContent]. It does not store the full [JournalistInfo]
 * but instead only the [recipientId] which is resolved upon loading by using the cached information
 * on the available journalists.
 */
internal data class StoredMessageThread(
    val recipientId: JournalistId,
    val messages: List<StoredMessage>,
) {

    /**
     * [maxSize] provides the maximum size of the returned [ByteArray]. If the serialization would
     * exceed this value a [java.nio.BufferOverflowException] is thrown.
     */
    @kotlin.jvm.Throws(BufferOverflowException::class)
    internal fun serialize(maxSize: Int): ByteArray {
        val buffer = ByteBuffer.allocate(maxSize)

        buffer.putLengthEncodedByteArray(recipientId.encodeToByteArray())
        buffer.putLengthEncodedByteArray(
            messages.serializeOrThrow(
                maxSize = buffer.remaining() - LENGTH_ENCODING_OVERHEAD,
                serializeElement = { it.serialize() }
            )
        )

        // only return the written bytes (we conservatively allocated more)
        return buffer.getWrittenBytes()
    }

    companion object {
        internal fun deserialize(bytes: ByteArray): StoredMessageThread {
            val buffer = ByteBuffer.wrap(bytes)

            val recipientId = buffer.getLengthEncodedByteArray().decodeToString()
            val messages = deserializeList(
                bytes = buffer.getLengthEncodedByteArray(),
                deserializeElement = { StoredMessage.deserialize(it) }
            )

            return StoredMessageThread(recipientId, messages)
        }
    }

    /**
     * Returns the most recent timestamp of the included messages
     * */
    internal fun mostRecentUpdate(): Instant {
        return messages.maxOf { it.timestamp }
    }

    /**
     * Returns a copy of the [StoredMessageThread] but with all messages older than [cutoff]
     * removed.
     */
    fun copyAndRemoveOlderMessages(cutoff: Instant): StoredMessageThread {
        val newMessages = messages.filter { it.timestamp >= cutoff }
        return this.copy(messages = newMessages)
    }
}

