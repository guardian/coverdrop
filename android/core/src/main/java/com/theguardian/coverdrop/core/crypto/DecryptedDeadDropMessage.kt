package com.theguardian.coverdrop.core.crypto

import com.theguardian.coverdrop.core.api.models.JournalistIdentity
import com.theguardian.coverdrop.core.generated.FLAG_J2U_MESSAGE_TYPE_HANDOVER
import com.theguardian.coverdrop.core.generated.FLAG_J2U_MESSAGE_TYPE_MESSAGE
import com.theguardian.coverdrop.core.generated.MAX_JOURNALIST_IDENTITY_LEN
import com.theguardian.coverdrop.core.models.JournalistId
import com.theguardian.coverdrop.core.models.PaddedCompressedString
import com.theguardian.coverdrop.core.utils.getRemainingAsByteArray
import java.nio.ByteBuffer
import java.time.Instant

/**
 * An abstract decrypted message from the dead drop that can be either a normal text message
 * [Text] or a hand-over command [Handover].
 */
internal sealed class DecryptedDeadDropMessage(
    val remoteId: JournalistId,
    val timestamp: Instant
) {
    companion object {
        fun parse(
            bytes: ByteArray,
            remoteId: JournalistId,
            timestamp: Instant
        ): DecryptedDeadDropMessage {
            val buffer = ByteBuffer.wrap(bytes)
            val typeFlag = buffer.get()
            val payload = buffer.getRemainingAsByteArray()

            return when (typeFlag) {
                FLAG_J2U_MESSAGE_TYPE_MESSAGE -> Text.parse(
                    bytes = payload,
                    remoteId = remoteId,
                    timestamp = timestamp
                )

                FLAG_J2U_MESSAGE_TYPE_HANDOVER -> Handover.parse(
                    bytes = payload,
                    remoteId = remoteId,
                    timestamp = timestamp
                )

                else -> Unknown.parse(remoteId = remoteId, timestamp = timestamp)
            }
        }
    }

    /**
     * A message with text from the journalist to the user.
     */
    internal class Text(
        remoteId: JournalistIdentity,
        timestamp: Instant,
        val message: String,
    ) : DecryptedDeadDropMessage(remoteId, timestamp) {
        companion object {
            fun parse(
                bytes: ByteArray,
                remoteId: JournalistId,
                timestamp: Instant
            ): Text {
                val paddedCompressedString = PaddedCompressedString(bytes)
                return Text(
                    remoteId = remoteId,
                    timestamp = timestamp,
                    message = paddedCompressedString.toPayloadString(),
                )
            }
        }
    }

    /**
     * A message with a hand-over flag set.
     */
    internal class Handover(
        remoteId: JournalistId,
        timestamp: Instant,
        val handoverTo: JournalistIdentity
    ) : DecryptedDeadDropMessage(remoteId, timestamp) {
        companion object {
            fun parse(
                bytes: ByteArray,
                remoteId: JournalistId,
                timestamp: Instant
            ): Handover {
                // the remainder of the message is expected to be 0x00 bytes
                val end = bytes.indexOfFirst { it == 0x00.toByte() }

                if (end in 0 until MAX_JOURNALIST_IDENTITY_LEN) {
                    return Handover(
                        remoteId,
                        timestamp,
                        bytes.slice(0 until end).toByteArray().decodeToString()
                    )
                } else {
                    throw IllegalArgumentException("failed parsing journalist identity (end=$end)")
                }
            }
        }
    }

    /**
     * A message of an unknown type (usually indicating forward protocol changes).
     */
    internal class Unknown(
        remoteId: JournalistId,
        timestamp: Instant,
    ) : DecryptedDeadDropMessage(remoteId, timestamp) {
        companion object {
            fun parse(
                remoteId: JournalistId,
                timestamp: Instant,
            ): Unknown {
                return Unknown(remoteId, timestamp)
            }
        }
    }
}
