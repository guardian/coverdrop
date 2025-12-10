package com.theguardian.coverdrop.core.generated

// DO NOT EDIT! This file is auto-generated from Rust code using the following command:
// `cargo run --bin admin generate-mobile-constants-files`.
//
// The Rust code is here: common/src/protocol/constants.rs

internal const val JOURNALIST_PROVISIONING_KEY_VALID_DURATION_SECONDS = 31_449_600
internal const val JOURNALIST_ID_KEY_VALID_DURATION_SECONDS = 4_838_400
internal const val JOURNALIST_MSG_KEY_VALID_DURATION_SECONDS = 1_209_600
internal const val COVERNODE_PROVISIONING_KEY_VALID_DURATION_SECONDS = 31_449_600
internal const val COVERNODE_ID_KEY_VALID_DURATION_SECONDS = 2_419_200
internal const val COVERNODE_MSG_KEY_VALID_DURATION_SECONDS = 1_209_600
internal const val USER_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN = 773
internal const val USER_TO_COVERNODE_MESSAGE_LEN = 597
internal const val USER_TO_JOURNALIST_ENCRYPTED_MESSAGE_LEN = 593
internal const val USER_TO_JOURNALIST_MESSAGE_LEN = 545
internal const val JOURNALIST_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN = 730
internal const val JOURNALIST_TO_COVERNODE_MESSAGE_LEN = 554
internal const val JOURNALIST_TO_USER_ENCRYPTED_MESSAGE_LEN = 553
internal const val JOURNALIST_TO_USER_MESSAGE_LEN = 513
internal const val MESSAGE_PADDING_LEN = 512
internal const val RECIPIENT_TAG_LEN = 4
internal const val MESSAGE_VALID_FOR_DURATION_IN_SECONDS = 1_209_600
internal const val MESSAGE_EXPIRY_WARNING_IN_SECONDS = 172_800
internal const val CLIENT_DEAD_DROP_CACHE_TTL_SECONDS = 1_209_600
internal const val CLIENT_DEFAULT_DOWNLOAD_RATE_SECONDS = 3600
internal const val CLIENT_STATUS_DOWNLOAD_RATE_SECONDS = 300
internal const val COVERNODE_WRAPPING_KEY_COUNT = 2
internal const val MAX_JOURNALIST_IDENTITY_LEN = 128
internal const val FLAG_J2U_MESSAGE_TYPE_MESSAGE: Byte = 0x00
internal const val FLAG_J2U_MESSAGE_TYPE_HANDOVER: Byte = 0x01
internal val RECIPIENT_TAG_BYTES_U2J_COVER = byteArrayOf(0x00, 0x00, 0x00, 0x00)

