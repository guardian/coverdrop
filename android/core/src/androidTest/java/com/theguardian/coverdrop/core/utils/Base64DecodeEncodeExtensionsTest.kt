package com.theguardian.coverdrop.core.utils


import com.google.common.truth.Truth.assertThat
import org.junit.Test

class Base64DecodeEncodeExtensionsTest {

    @Test
    fun testBase64EncodeDecode_whenEmpty_thenMatches() {
        val bytes = ByteArray(0)

        val encoded = bytes.base64Encode()
        assertThat(encoded).isEqualTo("")

        val decoded = encoded.base64Decode()
        assertThat(decoded).isEqualTo(bytes)
    }

    @Suppress("UsePropertyAccessSyntax")
    @Test
    fun testBase64EncodeDecode_whenNonEmpty_thenMatches() {
        val bytes = "hello".encodeToByteArray()

        val encoded = bytes.base64Encode()
        assertThat(encoded).isNotEmpty()

        val decoded = encoded.base64Decode()
        assertThat(decoded).isEqualTo(bytes)
    }
}
