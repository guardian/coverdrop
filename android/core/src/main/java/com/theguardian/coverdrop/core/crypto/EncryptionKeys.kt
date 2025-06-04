package com.theguardian.coverdrop.core.crypto

import androidx.annotation.VisibleForTesting
import com.goterl.lazysodium.SodiumAndroid
import com.goterl.lazysodium.interfaces.Box
import com.theguardian.coverdrop.core.utils.LENGTH_ENCODING_OVERHEAD
import com.theguardian.coverdrop.core.utils.checkLibSodiumSuccess
import com.theguardian.coverdrop.core.utils.getLengthEncodedByteArray
import com.theguardian.coverdrop.core.utils.hexDecode
import com.theguardian.coverdrop.core.utils.putLengthEncodedByteArray
import java.nio.ByteBuffer

internal const val X25519_PUBLIC_KEY_BYTES = Box.PUBLICKEYBYTES
internal const val X25519_SECRET_KEY_BYTES = Box.SECRETKEYBYTES

data class PublicEncryptionKey(internal val bytes: ByteArray) {
    init {
        require(bytes.size == X25519_PUBLIC_KEY_BYTES) { "bad key length" }
    }

    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (javaClass != other?.javaClass) return false

        other as PublicEncryptionKey
        return bytes.contentEquals(other.bytes)
    }

    override fun hashCode(): Int {
        return bytes.contentHashCode()
    }
}

internal data class SecretEncryptionKey(internal val bytes: ByteArray) {
    init {
        require(bytes.size == X25519_SECRET_KEY_BYTES) { "bad key length" }
    }

    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (javaClass != other?.javaClass) return false

        other as SecretEncryptionKey
        return bytes.contentEquals(other.bytes)
    }

    override fun hashCode(): Int {
        return bytes.contentHashCode()
    }
}

internal data class EncryptionKeyPair(
    internal val publicEncryptionKey: PublicEncryptionKey,
    internal val secretEncryptionKey: SecretEncryptionKey,
) {
    companion object {
        internal fun new(libSodium: SodiumAndroid): EncryptionKeyPair {
            val publicKey = ByteArray(X25519_PUBLIC_KEY_BYTES)
            val secretKey = ByteArray(X25519_SECRET_KEY_BYTES)
            val res = libSodium.crypto_box_keypair(
                /* publicKey = */ publicKey,
                /* secretKey = */ secretKey
            )
            checkLibSodiumSuccess(res)

            return EncryptionKeyPair(
                publicEncryptionKey = PublicEncryptionKey(publicKey),
                secretEncryptionKey = SecretEncryptionKey(secretKey)
            )
        }

        internal fun deserialize(bytes: ByteArray): EncryptionKeyPair {
            val buffer = ByteBuffer.wrap(bytes)

            return EncryptionKeyPair(
                PublicEncryptionKey(buffer.getLengthEncodedByteArray()),
                SecretEncryptionKey(buffer.getLengthEncodedByteArray()),
            )
        }

        @VisibleForTesting
        internal fun newFromHexStrings(publicKey: String, secretKey: String): EncryptionKeyPair {
            return EncryptionKeyPair(
                PublicEncryptionKey(publicKey.hexDecode()),
                SecretEncryptionKey(secretKey.hexDecode()),
            )
        }
    }

    internal fun serialize(): ByteArray {
        val buffer = ByteBuffer.allocate(
            LENGTH_ENCODING_OVERHEAD + X25519_PUBLIC_KEY_BYTES +
                    LENGTH_ENCODING_OVERHEAD + X25519_SECRET_KEY_BYTES
        )

        buffer.putLengthEncodedByteArray(publicEncryptionKey.bytes)
        buffer.putLengthEncodedByteArray(secretEncryptionKey.bytes)

        // return the entire buffer as we have allocated exactly the right number of bytes
        check(buffer.remaining() == 0)
        return buffer.array()
    }
}
