package com.theguardian.coverdrop.core.crypto

import com.goterl.lazysodium.SodiumAndroid
import com.goterl.lazysodium.interfaces.Sign
import com.theguardian.coverdrop.core.utils.checkLibSodiumSuccess
import com.theguardian.coverdrop.core.utils.hexDecode

// NOTE: the libSodium's ED25519 secret key is equivalent to the `ExpandedSecretKey` (64 bytes) in
// the Rust library . The seed bytes (32 bytes) are referred to as the `SecretKey` in the Rust
// library. To avoid confusion, we follow the Rust terminology here.

internal const val ED25519_PUBLIC_KEY_BYTES = Sign.ED25519_PUBLICKEYBYTES
internal const val ED25519_SECRET_KEY_BYTES = Sign.ED25519_SEEDBYTES
internal const val ED25519_EXPANDED_SECRET_KEY_BYTES = Sign.ED25519_SECRETKEYBYTES

internal data class SecretSigningKey(internal val bytes: ByteArray) {
    init {
        require(bytes.size == ED25519_SECRET_KEY_BYTES) { "bad key length" }
    }

    /**
     * Returns the expanded secret key that can be directly used with the libSodium operations.
     */
    internal fun getExpandedSecretKey(libSodium: SodiumAndroid): ByteArray {
        val publicKeyBytes = ByteArray(ED25519_PUBLIC_KEY_BYTES) // intentionally unused
        val secretKeyBytes = ByteArray(ED25519_EXPANDED_SECRET_KEY_BYTES)

        val res = libSodium.crypto_sign_seed_keypair(
            /* publicKey = */ publicKeyBytes,
            /* secretKey = */ secretKeyBytes,
            /* seed = */ this.bytes
        )
        checkLibSodiumSuccess(res)

        return secretKeyBytes
    }

    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (javaClass != other?.javaClass) return false

        other as SecretSigningKey
        return bytes.contentEquals(other.bytes)
    }

    override fun hashCode(): Int {
        return bytes.contentHashCode()
    }
}

data class PublicSigningKey(internal val bytes: ByteArray) {
    init {
        require(bytes.size == ED25519_PUBLIC_KEY_BYTES) { "bad key length" }
    }

    companion object {
        /**
         * Creates a new [PublicSigningKey] from a hex-encoded string. This method only checks that
         * the string has the correct length and is hex-encoded. It does not check that the bytes
         * represent a valid ED25519 public key.
         */
        fun fromHexEncodedString(hexEncodedString: String): PublicSigningKey {
            return PublicSigningKey(hexEncodedString.hexDecode())
        }
    }

    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (javaClass != other?.javaClass) return false

        other as PublicSigningKey
        return bytes.contentEquals(other.bytes)
    }

    override fun hashCode(): Int {
        return bytes.contentHashCode()
    }
}

internal data class SigningKeyPair(
    internal val publicSigningKey: PublicSigningKey,
    internal val secretSigningKey: SecretSigningKey,
) {
    companion object {
        internal fun new(libSodium: SodiumAndroid): SigningKeyPair {
            // generate key pair; the libSodium "secret key" will be an expanded secret key
            val publicKeyBytes = ByteArray(ED25519_PUBLIC_KEY_BYTES)
            val expandedSecretKeyBytes = ByteArray(ED25519_EXPANDED_SECRET_KEY_BYTES)
            val res = libSodium.crypto_sign_keypair(
                /* publicKey = */ publicKeyBytes,
                /* secretKey = */ expandedSecretKeyBytes
            )
            checkLibSodiumSuccess(res)

            // extract seed bytes from the expanded secret key
            val secretKeyBytes = ByteArray(ED25519_SECRET_KEY_BYTES)
            val res2 = libSodium.crypto_sign_ed25519_sk_to_seed(
                /* seed = */ secretKeyBytes,
                /* ed25519SecretKey = */ expandedSecretKeyBytes
            )
            checkLibSodiumSuccess(res2)

            return SigningKeyPair(
                publicSigningKey = PublicSigningKey(publicKeyBytes),
                secretSigningKey = SecretSigningKey(secretKeyBytes)
            )
        }
    }
}
