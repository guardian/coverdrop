package com.theguardian.coverdrop.core.crypto

import com.google.common.truth.Truth.assertThat
import com.theguardian.coverdrop.core.createLibSodium
import org.junit.Test


class SigningKeysTest {
    private val libSodium = createLibSodium()

    @Test
    fun testNew_whenCreatingTwoPairs_thenTheyAreDifferent() {
        val keyPairA = SigningKeyPair.new(libSodium)
        val keyPairB = SigningKeyPair.new(libSodium)

        assertThat(keyPairA).isNotEqualTo(keyPairB)
    }
}
