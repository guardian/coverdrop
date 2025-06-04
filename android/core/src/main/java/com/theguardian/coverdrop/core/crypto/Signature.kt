package com.theguardian.coverdrop.core.crypto

import com.goterl.lazysodium.SodiumAndroid
import com.goterl.lazysodium.interfaces.Sign
import com.sun.jna.Pointer
import com.theguardian.coverdrop.core.utils.checkLibSodiumSuccess
import java.security.SignatureException


internal data class Signature<T>(internal val bytes: ByteArray) {
    init {
        require(bytes.size == Sign.ED25519_BYTES) { "bad signature length" }
    }

    companion object {
        fun <T> sign(
            libSodium: SodiumAndroid,
            signingSk: SecretSigningKey,
            data: T,
        ): Signature<T> where T : Signable {
            // expand the key to match libSodium's expectations (see notes in `SigningKeys.kt`)
            val expandedSecretKeyBytes = signingSk.getExpandedSecretKey(libSodium)

            val message = data.asBytes()
            val signatureBytes = ByteArray(Sign.ED25519_BYTES)

            val res = libSodium.crypto_sign_detached(
                /* signature = */ signatureBytes,
                // `null` is correct: https://github.com/terl/lazysodium-android/issues/21
                /* sigLength = */ Pointer.NULL,
                /* message = */ message,
                /* messageLen = */ message.size.toLong(),
                /* secretKey = */ expandedSecretKeyBytes,
            )
            checkLibSodiumSuccess(res)

            return Signature(signatureBytes)
        }

        @Throws(SignatureException::class)
        fun <T> verifyOrThrow(
            libSodium: SodiumAndroid,
            signingPk: PublicSigningKey,
            data: T,
            signature: Signature<T>,
        ) where T : Signable {
            val message = data.asBytes()
            val signatureBytes = signature.bytes

            val res = libSodium.crypto_sign_verify_detached(
                /* signature = */ signatureBytes,
                /* message = */ message,
                /* messageLen = */ message.size.toLong(),
                /* publicKey = */ signingPk.bytes
            )
            if (res != 0) {
                throw SignatureException("bad signature")
            }
        }
    }

    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (javaClass != other?.javaClass) return false

        other as Signature<*>
        return bytes.contentEquals(other.bytes)
    }

    override fun hashCode(): Int {
        return bytes.contentHashCode()
    }

}
