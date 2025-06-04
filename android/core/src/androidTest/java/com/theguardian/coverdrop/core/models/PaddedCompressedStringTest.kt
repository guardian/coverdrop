package com.theguardian.coverdrop.core.models

import com.google.common.truth.Truth.assertThat
import com.theguardian.coverdrop.core.generated.MESSAGE_PADDING_LEN
import org.junit.Test
import java.nio.BufferOverflowException

class PaddedCompressedStringTest {
    @Test
    fun successfullyRoundTrip() {
        val expected = "hello world"

        val pcs = PaddedCompressedString.fromString(expected)
        assertThat(pcs.totalLength()).isEqualTo(MESSAGE_PADDING_LEN)

        val actual = pcs.toPayloadString()

        assertThat(actual).isEqualTo(expected)
    }

    @Test
    fun messageIsAlwaysTheSameSize() {
        val messages = listOf(
            "a",
            "this is a small message",
            "this is a longer message with a few extra words",
        )

        val targetLength = 512

        for (message in messages) {
            val pcs = PaddedCompressedString.fromString(message)
            assertThat(pcs.totalLength()).isEqualTo(targetLength)
        }
    }

    @Test(expected = BufferOverflowException::class)
    fun willErrorIfStringIsTooLong() {
        val message = """"Lorem ipsum dolor sit amet, consectetur adipiscing elit. Integer dolor 
            nulla, ornare et tristique imperdiet, dictum sit amet velit. Curabitur pharetra erat sed
            neque interdum, non mattis tortor auctor. Curabitur eu ipsum ac neque semper eleifend.
            Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus.
            Integer erat mi, ultrices nec arcu ut, sagittis sollicitudin est. In hac habitasse
            platea dictumst. Sed in efficitur elit. Curabitur nec commodo elit. Aliquam tincidunt
            rutrum nisl ut facilisis. Aenean ornare ut mauris eget lacinia. Mauris a felis quis orci
            auctor varius sit amet eget est. Curabitur a urna sit amet diam sagittis aliquet eget eu
            sapien. Curabitur a pharetra purus.
            Nulla facilisi. Suspendisse potenti. Morbi mollis aliquet sapien sed faucibus. Donec
            aliquam nibh nibh, ac faucibus felis aliquam at. Pellentesque egestas enim sem, eu
            tempor urna posuere eget. Cras fermentum commodo neque ac gravida."""

        PaddedCompressedString.fromString(message)
    }

    @Test(expected = IllegalArgumentException::class)
    fun testToPayloadString_whenCompressionRatioTooHigh_thenThrows() {
        val message = "a".repeat(10000)
        val pcs = PaddedCompressedString.fromString(message)
        pcs.toPayloadString()
    }

    @Test
    fun testPaddingLength_whenNonEmptyPayload_thenInExpectedRange() {
        val pcs = PaddedCompressedString.fromString("Hello World")
        val paddingLength = pcs.paddingLength()

        assertThat(paddingLength).isLessThan(pcs.totalLength() - HEADER_SIZE)
        assertThat(paddingLength).isAtLeast(0)
    }

    @Test
    fun testFillLevel_whenNonEmptyPayload_thenInExpectedRange() {
        val pcs = PaddedCompressedString.fromString("Hello World")
        val fillLevel = pcs.fillLevel()

        assertThat(fillLevel).isLessThan(1f)
        assertThat(fillLevel).isGreaterThan(0f)
    }

    @Test
    fun nondeterministicTestPaddingIsNonZero() {
        val pcs = PaddedCompressedString.fromString("")

        val suffix = pcs.bytes.takeLast(MESSAGE_PADDING_LEN - 100)
        assertThat(suffix.count()).isAtLeast(100)
        assertThat(suffix.count { it == 0.toByte() }).isLessThan(10)
    }
}
