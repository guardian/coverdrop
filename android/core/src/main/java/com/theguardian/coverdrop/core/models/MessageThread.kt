package com.theguardian.coverdrop.core.models

import java.time.Instant

/**
 * A message thread or conversation between the user and a recipient which might either be an
 * individual journalist or a team.
 */
data class MessageThread(
    val recipient: JournalistInfo,
    val messages: List<Message>,
) {
    /**
     * Returns the timestamp of the most recent message in the thread. If the thread is empty, we
     * return `null`.
     */
    fun mostRecentUpdate(): Instant? {
        return messages.maxOfOrNull { it.timestamp }
    }
}
