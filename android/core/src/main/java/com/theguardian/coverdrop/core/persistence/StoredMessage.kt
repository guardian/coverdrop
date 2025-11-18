package com.theguardian.coverdrop.core.persistence

import androidx.annotation.VisibleForTesting
import com.theguardian.coverdrop.core.crypto.PRIVATE_SENDING_QUEUE_HINT_SIZE_BYTES
import com.theguardian.coverdrop.core.crypto.PrivateSendingQueueHint
import com.theguardian.coverdrop.core.utils.LENGTH_ENCODING_OVERHEAD
import com.theguardian.coverdrop.core.utils.getByteArray
import com.theguardian.coverdrop.core.utils.getLengthEncodedByteArray
import com.theguardian.coverdrop.core.utils.putLengthEncodedByteArray
import java.nio.ByteBuffer
import java.time.Instant

internal enum class StoredMessageType(val flag: Byte) {
    SENT(0x00),
    RECEIVED_MESSAGE(0x01),
    RECEIVED_HANDOVER(0x02),
    RECEIVED_UNKNOWN(0x7F);

    companion object {
        fun fromFlagByte(flag: Byte): StoredMessageType {
            val value = StoredMessageType.values().firstOrNull { it.flag == flag }
            return requireNotNull(value) { "bad flag: $flag" }
        }
    }
}

/**
 * A single message as stored within [MailboxContent].
 */
internal data class StoredMessage(
    val timestamp: Instant,
    val payload: String,
    val type: StoredMessageType,
    val privateSendingQueueHint: PrivateSendingQueueHint = PrivateSendingQueueHint(
        ByteArray(
            PRIVATE_SENDING_QUEUE_HINT_SIZE_BYTES
        )
    ),
) {
    internal fun serialize(): ByteArray {
        val messageBytes = payload.encodeToByteArray()

        val buffer = ByteBuffer.allocate(
            Long.SIZE_BYTES +
                    (LENGTH_ENCODING_OVERHEAD + messageBytes.size) +
                    Byte.SIZE_BYTES +
                    PRIVATE_SENDING_QUEUE_HINT_SIZE_BYTES
        )

        buffer.putLong(timestamp.toEpochMilli())
        buffer.putLengthEncodedByteArray(messageBytes)
        buffer.put(type.flag)
        buffer.put(privateSendingQueueHint.bytes)

        // return the entire buffer as we have allocated exactly the right number of bytes
        check(buffer.remaining() == 0)
        return buffer.array()
    }

    companion object {
        internal fun deserialize(bytes: ByteArray): StoredMessage {
            val buffer = ByteBuffer.wrap(bytes)

            val timestamp = Instant.ofEpochMilli(buffer.getLong())
            val message = buffer.getLengthEncodedByteArray().decodeToString()
            val type = StoredMessageType.fromFlagByte(buffer.get())
            val privateSendingQueueHint = PrivateSendingQueueHint(
                bytes = buffer.getByteArray(PRIVATE_SENDING_QUEUE_HINT_SIZE_BYTES)
            )

            check(buffer.remaining() == 0)

            return StoredMessage(timestamp, message, type, privateSendingQueueHint)
        }

        fun local(
            timestamp: Instant,
            message: String,
            privateSendingQueueHint: PrivateSendingQueueHint,
        ) = StoredMessage(
            timestamp = timestamp,
            payload = message,
            type = StoredMessageType.SENT,
            privateSendingQueueHint = privateSendingQueueHint
        )


        fun remote(timestamp: Instant, message: String) = StoredMessage(
            timestamp = timestamp,
            payload = message,
            type = StoredMessageType.RECEIVED_MESSAGE,
        )

        fun remoteHandover(timestamp: Instant, remoteId: String) = StoredMessage(
            timestamp = timestamp,
            payload = remoteId,
            type = StoredMessageType.RECEIVED_HANDOVER,
        )

        fun remoteUnknown(timestamp: Instant) = StoredMessage(
            timestamp = timestamp,
            payload = "",
            type = StoredMessageType.RECEIVED_UNKNOWN,
        )

        @VisibleForTesting
        internal fun localForTest(timestamp: Instant, message: String) = StoredMessage(
            timestamp = timestamp,
            payload = message,
            type = StoredMessageType.RECEIVED_MESSAGE,
        )
    }

    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (javaClass != other?.javaClass) return false

        other as StoredMessage

        // Using the epoch second timestamp here is important to correctly identify duplicates that
        // are serialized and deserialized with different precision
        if (timestamp.epochSecond != other.timestamp.epochSecond) return false

        if (payload != other.payload) return false
        if (type != other.type) return false
        if (privateSendingQueueHint != other.privateSendingQueueHint) return false

        return true
    }

    override fun hashCode(): Int {
        // Using the epoch second timestamp here is important to correctly identify duplicates that
        // are serialized and deserialized with different precision
        var result = timestamp.epochSecond.hashCode()

        result = 31 * result + payload.hashCode()
        result = 31 * result + type.hashCode()
        result = 31 * result + privateSendingQueueHint.hashCode()
        return result
    }
}
