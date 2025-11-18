package com.theguardian.coverdrop.core.models

import androidx.annotation.VisibleForTesting
import com.theguardian.coverdrop.core.generated.MESSAGE_EXPIRY_WARNING_IN_SECONDS
import com.theguardian.coverdrop.core.generated.MESSAGE_VALID_FOR_DURATION_IN_SECONDS
import com.theguardian.coverdrop.core.persistence.StoredMessage
import com.theguardian.coverdrop.core.persistence.StoredMessageType
import com.theguardian.coverdrop.core.utils.DefaultClock
import java.time.Instant
import java.time.temporal.ChronoUnit
import kotlin.math.ceil

/** See [Message.getExpiryState]. */
sealed class ExpiryState {
    object Fresh : ExpiryState()
    data class SoonToBeExpired(val expiresAt: Instant) : ExpiryState() {
        fun getTimeRemainingInHours(now: Instant): Long {
            val seconds = now.until(expiresAt, ChronoUnit.SECONDS)
            val hours = ceil(seconds.toDouble() / 3600).toLong()
            return hours
        }
    }

    object Expired : ExpiryState()
}

/**
 * An individual message in a [MessageThread].
 */
sealed class Message(
    /**
     * The [Instant] when this message was first registered by this device. For messages from the
     * user, this is the sent time. For messages from the journalist, this is the time when it was
     * first decrypted.
     */
    val timestamp: Instant,
) {
    fun isFromRemote(): Boolean {
        return when (this) {
            // pending and sent messages are always from the user
            is Pending -> false
            is Sent -> false
            // received hand-over and text messages are from journalists
            is Handover -> true
            is Received -> true
            // unknown messages are likely from remote as they are usually an artifact of protocol
            // updates on the server-side
            is Unknown -> true
        }
    }

    /**
     * Messages are either fresh, soon to be expired (i.e. expiry time is within the next 48 hours),
     * or expired (i.e. expiry time is in the past). Expired messages are deleted on logout, but
     * displayed in the UI one more time.
     */
    fun getExpiryState(now: Instant): ExpiryState {
        val beforeThisIsExpired = now
            .minusSeconds(MESSAGE_VALID_FOR_DURATION_IN_SECONDS.toLong())

        val beforeThisIsSoonExpiring = beforeThisIsExpired
            .plusSeconds(MESSAGE_EXPIRY_WARNING_IN_SECONDS.toLong())

        return if (timestamp.isAfter(beforeThisIsSoonExpiring)) {
            ExpiryState.Fresh
        } else if (timestamp.isAfter(beforeThisIsExpired)) {
            val expiresAt = timestamp.plusSeconds(MESSAGE_VALID_FOR_DURATION_IN_SECONDS.toLong())
            ExpiryState.SoonToBeExpired(expiresAt = expiresAt)
        } else {
            ExpiryState.Expired
        }
    }

    companion object {
        internal fun fromStored(storedMessage: StoredMessage, isPending: Boolean): Message {
            return when (storedMessage.type) {
                StoredMessageType.SENT -> when (isPending) {
                    true -> Pending(storedMessage.payload, storedMessage.timestamp)
                    false -> Sent(storedMessage.payload, storedMessage.timestamp)
                }

                StoredMessageType.RECEIVED_MESSAGE -> Received(
                    storedMessage.payload,
                    storedMessage.timestamp
                )

                StoredMessageType.RECEIVED_HANDOVER -> Handover(
                    storedMessage.payload,
                    storedMessage.timestamp
                )

                StoredMessageType.RECEIVED_UNKNOWN -> Unknown(storedMessage.timestamp)
            }
        }

        @VisibleForTesting
        fun pending(message: String, timestamp: Instant = DefaultClock().now()) =
            Pending(message, timestamp)

        @VisibleForTesting
        fun sent(message: String, timestamp: Instant = DefaultClock().now()) =
            Sent(message, timestamp)

        @VisibleForTesting
        fun received(message: String, timestamp: Instant = DefaultClock().now()) =
            Received(message, timestamp)
    }

    /**
     * A message received from a remote party (i.e. journalist).
     */
    class Received(val message: String, timestamp: Instant) : Message(timestamp)

    /**
     * A message that has been sent, but has not yet cleared the outgoing sending queue.
     */
    class Pending(val message: String, timestamp: Instant) : Message(timestamp)

    /**
     * A message that has been sent and is not in the outgoing sending queue.
     */
    class Sent(val message: String, timestamp: Instant) : Message(timestamp)

    /**
     * A handover command received from a remote party (i.e. journalist).
     */
    class Handover(val handoverTo: JournalistId, timestamp: Instant) : Message(timestamp)

    /**
     * Unknown message type (e.g. introduced in a later protocol version that this client does
     * not support)
     */
    class Unknown(timestamp: Instant) : Message(timestamp)
}
