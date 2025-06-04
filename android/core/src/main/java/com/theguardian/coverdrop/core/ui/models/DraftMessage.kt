package com.theguardian.coverdrop.core.ui.models

import com.theguardian.coverdrop.core.models.PaddedCompressedString
import java.nio.BufferOverflowException

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
     */
    fun getFillLimit(): Float {
        return try {
            val pcs = PaddedCompressedString.fromString(text)
            pcs.fillLevel()
        } catch (e: BufferOverflowException) {
            2f
        }
    }
}
