package com.theguardian.coverdrop.ui.utils

import android.text.format.DateUtils
import java.text.SimpleDateFormat
import java.time.Instant
import java.util.Date

/**
 * Formats a timestamp into a human-friendly time string, e.g. "5 minutes ago" to be used in the UI
 * when displaying a message or thread. Everything more recent than 12 hours will be displayed
 * as a relative time. Everything more recent than 1 minute will be displayed as with a
 * customizable "just now" string. Everything else will be displayed as an absolute time.
 *
 * This method's output depends on the user's locale and timezone.
 *
 * @param forceAbsoluteTime if true, the time will always be displayed as an absolute time
 * @param justNowString the string to display when the timestamp is within the last minute
 */
internal fun humanFriendlyMessageTimeString(
    timestamp: Instant,
    now: Instant,
    forceAbsoluteTime: Boolean = false,
    justNowString: String = "Just now"
): String {
    val timePassed = now.toEpochMilli() - timestamp.toEpochMilli()
    val isInTheFuture = timePassed < 0
    val isLongTimeAgo = timePassed > 12 * DateUtils.HOUR_IN_MILLIS
    val isJustNow = timePassed < DateUtils.MINUTE_IN_MILLIS

    return if (forceAbsoluteTime || isInTheFuture || isLongTimeAgo) {
        // if the time is in the future or more than 12 hours in the past, return the absolute time
        val formatter = SimpleDateFormat.getDateTimeInstance(
            /* dateStyle = */ SimpleDateFormat.MEDIUM,
            /* timeStyle = */ SimpleDateFormat.SHORT
        )
        formatter.format(Date(timestamp.toEpochMilli()))
    } else if (isJustNow) {
        // if the time is just now, return a custom string
        justNowString
    } else {
        // otherwise return the relative time
        DateUtils.getRelativeTimeSpanString(
            /* time = */ timestamp.toEpochMilli(),
            /* now = */ now.toEpochMilli(),
            /* minResolution = */ DateUtils.MINUTE_IN_MILLIS,
            /* flags = */ 0
        ).toString()
    }
}
