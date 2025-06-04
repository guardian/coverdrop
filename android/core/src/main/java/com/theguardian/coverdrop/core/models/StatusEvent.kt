package com.theguardian.coverdrop.core.models

import com.theguardian.coverdrop.core.api.models.SystemStatus

/**
 * Information for the CoverDrop entry screen on whether the services are currently available or
 * not. Where applicable, the [description] contains more information that can be shown to the
 * user.
 *
 * If the [isAvailable] flag is false, the UI should show an error and not allow the
 * user to create a new session or access an existing one.
 */
data class StatusEvent(
    val status: SystemStatus,
    val isAvailable: Boolean,
    val description: String,
)

val ErrorDuringInitialization = StatusEvent(
    status = SystemStatus.UNAVAILABLE,
    isAvailable = false,
    description = "Initialization failed due to a configuration error or network issues."
)
