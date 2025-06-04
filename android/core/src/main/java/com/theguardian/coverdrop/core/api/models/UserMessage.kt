package com.theguardian.coverdrop.core.api.models

import androidx.annotation.Keep

/**
 * API model for a `Message<EncryptedUserToCoverNodeMessage>`.
 */
@Keep // required to survive R8
data class UserMessage(
    val data: String,
)
