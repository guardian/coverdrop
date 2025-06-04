package com.theguardian.coverdrop.core.crypto

import androidx.annotation.VisibleForTesting
import javax.crypto.Cipher
import javax.crypto.spec.IvParameterSpec
import javax.crypto.spec.SecretKeySpec


/**
 * Abstraction of the AES-256 cipher providing authenticated (GCM) encryption and decryption methods.
 */
object AesCipher {

    @VisibleForTesting
    internal const val AES_GCM_TAG_LEN_BYTES = 16

    /**
     * Authenticated encryption using AES-256-GCM.
     */
    fun encryptAuthenticatedGcm(key: AesKey, data: ByteArray): AesCipherResult {
        val cipher = Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(Cipher.ENCRYPT_MODE, SecretKeySpec(key.bytes, "AES"))

        return AesCipherResult(AesIv(cipher.iv), cipher.doFinal(data))
    }

    /**
     * Authenticated decryption using AES-256-GCM.
     *
     * @throws javax.crypto.AEADBadTagException if the authentication tag does not match
     */
    fun decryptAuthenticatedGcm(key: AesKey, aesCipherResult: AesCipherResult): ByteArray {
        require(aesCipherResult.encryptedPayload.size >= AES_GCM_TAG_LEN_BYTES)

        val cipher = Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(
            Cipher.DECRYPT_MODE,
            SecretKeySpec(key.bytes, "AES"),
            IvParameterSpec(aesCipherResult.iv.bytes)
        )

        try {
            return cipher.doFinal(aesCipherResult.encryptedPayload)
        } catch (e: javax.crypto.BadPaddingException) {
            // Older Android APIs will throw the more general `BadPaddingException`. Since we are
            // not expecting any padding, we can throw the more specific `AEADBadTagException` sub
            // class instead.
            throw javax.crypto.AEADBadTagException(e.message)
        }
    }
}


/**
 * Wrapper of an AES-256 key.
 */
data class AesKey(val bytes: ByteArray) {
    init {
        require(bytes.size == AES_KEY_LEN_BYTES)
    }

    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (javaClass != other?.javaClass) return false

        other as AesIv
        return bytes.contentEquals(other.bytes)
    }

    override fun hashCode(): Int {
        return bytes.contentHashCode()
    }

    companion object {
        internal const val AES_KEY_LEN_BYTES = 32
    }
}


/**
 * Wrapper of a IV used with AES. Depending on the implementation this is 16 bytes long (required
 * for the OpenSSL implementation in older Android versions) or 12 bytes long (as required by some
 * StrongBox implementations).
 */
data class AesIv(val bytes: ByteArray) {
    init {
        require(bytes.size == AES_IV_LEN_12_BYTES || bytes.size == AES_IV_LEN_16_BYTES) { "bad IV len" }
    }

    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (javaClass != other?.javaClass) return false

        other as AesIv
        return bytes.contentEquals(other.bytes)
    }

    override fun hashCode(): Int {
        return bytes.contentHashCode()
    }

    companion object {
        internal const val AES_IV_LEN_12_BYTES = 12
        internal const val AES_IV_LEN_16_BYTES = 16
    }
}

/**
 * Wrapper of an AES encryption result containing both the IV and the ciphertext.
 */
data class AesCipherResult(val iv: AesIv, val encryptedPayload: ByteArray) {

    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (javaClass != other?.javaClass) return false

        other as AesCipherResult

        if (iv != other.iv) return false
        if (!encryptedPayload.contentEquals(other.encryptedPayload)) return false

        return true
    }

    override fun hashCode(): Int {
        var result = iv.hashCode()
        result = 31 * result + encryptedPayload.contentHashCode()
        return result
    }
}
