package com.theguardian.coverdrop.ui.utils

import android.content.Context

/**
 * A message to be shown to the user in the UI.
 *
 * @param messageResId The message to be shown.
 * @param isFatal Whether the message is fatal and should prevent the user from proceeding.
 */
data class UiErrorMessage(val messageResId: Int, val isFatal: Boolean) {
    fun getString(context: Context): String = context.getString(messageResId)

}

/**
 * Whether the UI should be enabled. It should be enabled when there is no error or
 * the error is not fatal.
 */
fun UiErrorMessage?.shouldUiBeEnabled(): Boolean {
    return this == null || !this.isFatal
}
