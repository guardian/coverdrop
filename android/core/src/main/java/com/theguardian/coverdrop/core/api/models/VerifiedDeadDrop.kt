package com.theguardian.coverdrop.core.api.models

import com.theguardian.coverdrop.core.crypto.EncryptableVector
import com.theguardian.coverdrop.core.crypto.TwoPartyBox
import java.time.Instant

internal typealias VerifiedDeadDrops = List<VerifiedDeadDrop>

internal data class VerifiedDeadDrop(
    val id: Int,
    val createdAt: Instant,
    val messages: List<TwoPartyBox<EncryptableVector>>,
)
