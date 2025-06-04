package com.theguardian.coverdrop.core.utils

import com.google.common.truth.Truth.assertThat
import org.junit.Test


class CharArrayExtensionsTest {

    @Test
    fun testToByteArray_utf8_whenEncodingFromSameString_thenResultsAreEqualToDirectEncode() {
        val s = "Hello こんにちは"

        val directEncoded = s.toByteArray(Charsets.UTF_8)
        val charArrayEncoded = s.toCharArray().toByteArray(Charsets.UTF_8)
        assertThat(charArrayEncoded).isEqualTo(directEncoded)
    }

    @Test
    fun testToByteArray_utf16_whenEncodingFromSameString_thenResultsAreEqualToDirectEncode() {
        val s = "Hello こんにちは"

        val directEncoded = s.toByteArray(Charsets.UTF_16)
        val charArrayEncoded = s.toCharArray().toByteArray(Charsets.UTF_16)
        assertThat(charArrayEncoded).isEqualTo(directEncoded)
    }
}
