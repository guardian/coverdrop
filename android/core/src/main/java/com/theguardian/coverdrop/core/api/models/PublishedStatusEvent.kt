package com.theguardian.coverdrop.core.api.models

import androidx.annotation.Keep
import com.google.gson.annotations.SerializedName
import java.time.Instant

enum class SystemStatus {
    AVAILABLE,
    UNAVAILABLE,
    DEGRADED_PERFORMANCE,
    SCHEDULED_MAINTENANCE,
    NO_INFORMATION,
}

/**
 * API model for a `StatusEvent`.
 */
@Keep // required to survive R8
data class PublishedStatusEvent(
    @SerializedName("status")
    val status: String,

    @SerializedName("is_available")
    val isAvailable: Boolean,

    @SerializedName("description")
    val description: String,

    @SerializedName("timestamp")
    val timestamp: Instant,
) {
    fun getStatus(): SystemStatus {
        return SystemStatus.entries.firstOrNull { status == it.name }
            ?: throw IllegalArgumentException("unknown status: $status")
    }
}
