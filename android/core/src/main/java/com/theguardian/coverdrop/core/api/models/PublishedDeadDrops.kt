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

    @SerializedName("signature")
    val signature: String,
)

@Keep // required to survive R8
data class PublishedJournalistToUserDeadDropsList(
    @SerializedName("dead_drops")
    val deadDrops: List<PublishedJournalistToUserDeadDrop>,
)
