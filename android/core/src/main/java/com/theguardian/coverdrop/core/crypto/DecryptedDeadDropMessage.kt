package com.theguardian.coverdrop.core.crypto

import com.theguardian.coverdrop.core.api.models.JournalistIdentity
import com.theguardian.coverdrop.core.generated.FLAG_J2U_MESSAGE_TYPE_MESSAGE
import com.theguardian.coverdrop.core.models.JournalistId
import com.theguardian.coverdrop.core.models.PaddedCompressedString
import com.theguardian.coverdrop.core.utils.getRemainingAsByteArray
import java.nio.ByteBuffer
import java.time.Instant

/**
 * An abstract decrypted message from the dead drop that can be either a normal text message
 * [Text] or an [Unknown] message type.
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

                // this includes the deprecated handover flag (0x01) which must not
                // trigger any logic
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
