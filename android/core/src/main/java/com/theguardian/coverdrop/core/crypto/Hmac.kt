package com.theguardian.coverdrop.core.crypto

import javax.crypto.Mac
import javax.crypto.spec.SecretKeySpec

private const val ALGORITHM_NAME_HMAC_SHA256 = "HmacSHA256"

/**
 * Computes HMAC-SHA256 of the given [message] keyed with [secret]. The result is expected to be
 * indistinguishable from random without knowledge of [secret].
 *
 * @return A 32 byte array containing the HMAC-SHA256 of [message] keyed with [secret]
 */
fun hmacSha256(secret: ByteArray, message: ByteArray): ByteArray {
    val hmac = Mac.getInstance(ALGORITHM_NAME_HMAC_SHA256)
    hmac.init(SecretKeySpec(secret, ALGORITHM_NAME_HMAC_SHA256))
    return hmac.doFinal(message)
}
