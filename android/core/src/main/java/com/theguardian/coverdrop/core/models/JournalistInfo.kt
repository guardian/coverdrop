package com.theguardian.coverdrop.core.models

typealias JournalistId = String

typealias JournalistTag = String

data class JournalistInfo(
    val id: JournalistId,
    val displayName: String,
    val sortName: String = displayName,
    val description: String,
    val isTeam: Boolean,
    val tag: JournalistTag,
    val visibility: JournalistVisibility,
)

enum class JournalistVisibility {
    VISIBLE,

    /**
     * This journalist should not be shown in the UI, because the server has marked them as
     * `HIDDEN_FROM_UI`. This is usually done when temporarily deactivating a journalist profile.
     */
    HIDDEN,
}
