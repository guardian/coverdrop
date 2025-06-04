package com.theguardian.coverdrop.core.api.models

import androidx.annotation.Keep
import com.google.gson.annotations.SerializedName
import java.time.Instant

@Keep // required to survive R8
data class PublishedJournalistToUserDeadDrop(
    @SerializedName("id")
    val id: Int,

    @SerializedName("created_at")
    val createdAt: Instant,

    @SerializedName("data")
    val data: String,

    @Deprecated("Replaced with signature that also covers the created_at field")
    @SerializedName("cert")
    val cert: String,

    // While there are still dead-drops around with a signature (e.g. the app might read from cache
    // long after the backend migrated), we keep the signature optional for now. This should be
    // removed eventually, to enforce the new signature check. #2998
    @SerializedName("signature")
    val signature: String?,
)

@Keep // required to survive R8
data class PublishedJournalistToUserDeadDropsList(
    @SerializedName("dead_drops")
    val deadDrops: List<PublishedJournalistToUserDeadDrop>,
)
