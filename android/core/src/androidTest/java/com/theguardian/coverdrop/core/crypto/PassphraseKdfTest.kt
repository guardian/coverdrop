package com.theguardian.coverdrop.core.crypto

import com.google.common.truth.Truth.assertThat
import com.theguardian.coverdrop.core.createLibSodium
import com.theguardian.coverdrop.core.utils.hexDecode
import com.theguardian.coverdrop.core.utils.hexEncode
import org.junit.Test


private const val TEST_KEY_LENGTH: Int = 32
private val TEST_SALT = "COVERDROPKDFSALT".toByteArray()

class PassphraseKdfTest {
    private val libSodium = createLibSodium()

    /**
     * This test ensures that potential changes to encoding (i.e. transforming strings to bytes) are
     * backwards compatible. Do not change the `expected` value unless you have added migration
     * code.
     */
    @Test
    fun testDeriveKeyFromString_whenGivenPassword_thenReturnsExpected() {
        val hash = PassphraseKdf.deriveKeyFromString(
            libSodium = libSodium,
            passphrase = "password".toCharArray(),
            salt = TEST_SALT,
            keyLengthInBytes = TEST_KEY_LENGTH,
            params = PassphraseKdfParameters.INTERACTIVE,
        )
        val expected = "20650A20003AAF0248884D17E7DEC18240714F91A729A5025B858BF72D4FD0EA"
        assertThat(hash).isEqualTo(expected.hexDecode())
    }

    /**
     * This test ensures that potential changes to encoding (i.e. transforming strings to bytes) are
     * backwards compatible. Do not change the `expected` value unless you have added migration
     * code.
     */
    @Test
    fun testDeriveKeyFromString_whenGivenHigherCosts_thenReturnsExpected() {
        val hash = PassphraseKdf.deriveKeyFromString(
            libSodium = libSodium,
            passphrase = "password".toCharArray(),
            salt = TEST_SALT,
            keyLengthInBytes = TEST_KEY_LENGTH,
            params = PassphraseKdfParameters.HIGH,
        )
        val expected = "66be2c467f0dc9d43a3761f48f79980b0b5cab8a13690bb62929d08ac02775e6"
        assertThat(hash.hexEncode()).isEqualTo(expected)
    }

    @Test
    fun testDeriveKeyFromString_whenPasswordDifferent_thenHashDifferent() {
        val hashA = PassphraseKdf.deriveKeyFromString(
            libSodium = libSodium,
            passphrase = "password_a".toCharArray(),
            salt = TEST_SALT,
            keyLengthInBytes = TEST_KEY_LENGTH,
            params = PassphraseKdfParameters.INTERACTIVE,
        )
        val hashB = PassphraseKdf.deriveKeyFromString(
            libSodium = libSodium,
            passphrase = "password_b".toCharArray(),
            salt = TEST_SALT,
            keyLengthInBytes = TEST_KEY_LENGTH,
            params = PassphraseKdfParameters.INTERACTIVE,
        )

        assertThat(hashA).isNotEqualTo(hashB)
    }

    @Test
    fun testDeriveKeyFromString_whenSaltDifferent_thenHashDifferent() {
        val hashA = PassphraseKdf.deriveKeyFromString(
            libSodium = libSodium,
            passphrase = "password".toCharArray(),
            salt = "COVERDROPKDFSALT".toByteArray(),
            keyLengthInBytes = TEST_KEY_LENGTH,
            params = PassphraseKdfParameters.INTERACTIVE,
        )
        val hashB = PassphraseKdf.deriveKeyFromString(
            libSodium = libSodium,
            passphrase = "password".toCharArray(),
            salt = "MOVERDROPKDFSALT".toByteArray(),
            keyLengthInBytes = TEST_KEY_LENGTH,
            params = PassphraseKdfParameters.INTERACTIVE,
        )

        assertThat(hashA).isNotEqualTo(hashB)
    }
}
