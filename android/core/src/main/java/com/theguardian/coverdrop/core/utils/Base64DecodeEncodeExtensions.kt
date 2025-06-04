package com.theguardian.coverdrop.core.utils

import android.util.Base64

/**
 * Decodes a Base64 [String] as a [ByteArray].
 */
internal fun String.base64Decode(): ByteArray {
    return Base64.decode(this, Base64.NO_PADDING)
}

/**
 * Encodes a [ByteArray] as a Base64 [String].
 */
internal fun ByteArray.base64Encode(): String {
    return Base64.encodeToString(this, Base64.NO_PADDING or Base64.NO_WRAP)
}
