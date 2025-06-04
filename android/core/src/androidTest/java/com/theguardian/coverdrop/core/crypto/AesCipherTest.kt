package com.theguardian.coverdrop.core.crypto

import com.google.common.truth.Truth.assertThat
import com.theguardian.coverdrop.core.crypto.AesCipher.AES_GCM_TAG_LEN_BYTES
import org.junit.Test


class AesCipherTest {
    private val key = AesKey("secret__secret__secret__secret__".toByteArray())
    private val data = "hello".toByteArray()

    @Test
    fun testAesGcmEncryptDecrypt_whenGivenSameIvAndKey_thenResultDecryptsCorrectly() {
        val aesCipherResult = AesCipher.encryptAuthenticatedGcm(key, data)
        assertThat(aesCipherResult.encryptedPayload.size).isEqualTo(data.size + AES_GCM_TAG_LEN_BYTES)

        val actual = AesCipher.decryptAuthenticatedGcm(key, aesCipherResult)
        assertThat(actual).isEqualTo(data)
    }

    @Test(expected = javax.crypto.AEADBadTagException::class)
    fun testAesGcmEncryptDecrypt_whenGivenWrongIv_thenDecryptThrows() {
        val aesCipherResult = AesCipher.encryptAuthenticatedGcm(key, data)
        assertThat(aesCipherResult.encryptedPayload.size).isEqualTo(data.size + AES_GCM_TAG_LEN_BYTES)

        val iv2 = "__vi__vi__vi__vi".toByteArray()
        val aesCipherResult2 = AesCipherResult(AesIv(iv2), aesCipherResult.encryptedPayload)

        AesCipher.decryptAuthenticatedGcm(key, aesCipherResult2)
    }

    @Test(expected = javax.crypto.AEADBadTagException::class)
    fun testAesGcmEncryptDecrypt_whenGivenWrongKey_thenDecryptThrows() {
        val aesCipherResult = AesCipher.encryptAuthenticatedGcm(key, data)
        assertThat(aesCipherResult.encryptedPayload.size).isEqualTo(data.size + AES_GCM_TAG_LEN_BYTES)

        val key2 = AesKey("__terces__terces__terces__terces".toByteArray())
        AesCipher.decryptAuthenticatedGcm(key2, aesCipherResult)
    }
}
