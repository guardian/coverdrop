package com.theguardian.coverdrop.core.ui.models

import com.google.common.truth.Truth.assertThat
import org.junit.Test
import kotlin.random.Random


class DraftMessageTest {

    @Test
    fun testFillLimit_whenEmptyOrTooLong_thenExtremeValues() {
        val empty = DraftMessage("")
        assertThat(empty.getFillLevel()).isEqualTo(0f)

        val randomString = Random(42).nextBytes(1000).decodeToString()
        val tooLong = DraftMessage(randomString)
        assertThat(tooLong.getFillLevel()).isEqualTo(2f)
    }

    @Test
    fun testFillLimit_whenAddingMoreCharacters_thenIncreasesRoughlyMonotonically() {
        var currentText = ""
        var previousFillLevel = 0f
        val random = Random(42)

        while (currentText.length < 1000) {
            val draftMessage = DraftMessage(currentText)

            val fillLimit = draftMessage.getFillLevel()
            assertThat(fillLimit).isAtLeast(0f)
            assertThat(fillLimit).isAtMost(2f)

            // Roughly monotonically increasing (with GZip a long strings might actually
            // compress to a slightly smaller size than its own prefix)
            assertThat(fillLimit).isAtLeast(previousFillLevel - 0.01f)

            // Extend the text with a random printable ASCII character
            val newChar = random.nextInt(32, 127).toChar()
            currentText += newChar

            previousFillLevel = fillLimit
        }
    }
}
