package com.theguardian.coverdrop.core.crypto

import com.google.common.truth.Truth.assertThat
import com.theguardian.coverdrop.core.createLibSodium
import com.theguardian.coverdrop.core.utils.hexDecode
import org.junit.Test


class HumanKeyDigestTest {
    private val libSodium = createLibSodium()

    @Test
    fun testHumanKeyDigest_whenGivenSomeKey_thenResultMatchesExpectations() {
        val keyBytes =
            "c941a9beed1c8c945c27b150b5aa725a6366f71900a5e93607ba93254fe8d585".hexDecode()
        val key = PublicSigningKey(keyBytes)

        val actual = getHumanReadableDigest(libSodium, key)
        val expected = "jdiH4c 9DO9cT kefiCh OXoQ"
        assertThat(actual).isEqualTo(expected)
    }
}
