package com.theguardian.coverdrop.core.models

import java.text.SimpleDateFormat
import java.time.Instant
import java.util.Date
import java.util.Locale

/**
 * Debug information that can be used to diagnose issues with the app. Typically tucked away at
 * the bottom of the about screen.
 */
data class DebugContext(
    val lastUpdatePublicKeys: Instant?,
    val lastUpdateDeadDrops: Instant?,
    val lastBackgroundTry: Instant?,
    val lastBackgroundSend: Instant?,
    val hashedOrgKey: String?
) {
    override fun toString(): String {
        return "public keys: ${prettyInstantString(lastUpdatePublicKeys)}\n" +
                "dead drops:  ${prettyInstantString(lastUpdateDeadDrops)}\n" +
                "bg success:  ${prettyInstantString(lastBackgroundSend)}\n" +
                "bg trigger:  ${prettyInstantString(lastBackgroundTry)}\n" +
                "root: $hashedOrgKey"
    }

    private fun prettyInstantString(instant: Instant?): String {
        if (instant == null) {
            return "never"
        }

        val formatter = SimpleDateFormat("yyyy-MM-dd+HH:mm:ss", Locale.UK)
        return formatter.format(Date.from(instant))
    }
}
