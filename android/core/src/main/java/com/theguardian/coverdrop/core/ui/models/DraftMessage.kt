package com.theguardian.coverdrop.core.ui.models

import com.theguardian.coverdrop.core.models.PaddedCompressedString
import java.nio.BufferOverflowException
import kotlin.math.min

/**
 * A message that is being composed by the user and has not been sent yet.
 */
data class DraftMessage(val text: String) {

    /**
     * Returns without throwing if the message is valid, otherwise throws an exception.
     *
     * @throws [BufferOverflowException] if the message is too long
     */
    fun validateOrThrow() {
        PaddedCompressedString.fromString(text)
    }

    /**
     * Returns the fill limit of the message as a percentage of the maximum message length. If
     * the message is empty, this value will be close to 0f and as the message is growing the value
     * will approach 1f. If the message is too long, this value will be greater than 1f.
     *
     * This function will smooth the underlying `PaddedCompressedString#fillLevel` method to avoid
     * the value jumping abruptly for the first character typed.
     */
    fun getFillLevel(): Float {
        return try {
            val pcs = PaddedCompressedString.fromString(text)

            if (pcs.bytes.isEmpty()) {
                return 0f
            }

            val rawFillLevel = pcs.fillLevel()

            // The initial ramp fill level is always overestimating the fill level, as it
            // expects a non-compressed string.
            val initialRampFillLevel = text.count().toFloat() / pcs.totalLength().toFloat()

            // Return the smoothed (conservatively overestimating linear approximation) until
            // that estimation exceeds the raw fill level.
            return min(rawFillLevel, initialRampFillLevel)
        } catch (_: BufferOverflowException) {
            2f
        }
    }
}
