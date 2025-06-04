package com.theguardian.coverdrop.core.crypto

import com.google.common.truth.Truth.assertThat
import com.theguardian.coverdrop.core.utils.hexDecode
import org.junit.Test


class HmacTest {

    @Test
    fun testHmacSha256_whenGivenRfc4231Testcase1_thenMatchesExpectedOutput() {
        // see: https://datatracker.ietf.org/doc/html/rfc4231#section-4.2
        val key = "0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b".hexDecode()
        val data = "4869205468657265".hexDecode()
        val expected =
            "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7".hexDecode()

        val actual = hmacSha256(key, data)

        assertThat(actual).isEqualTo(expected)
    }

}
