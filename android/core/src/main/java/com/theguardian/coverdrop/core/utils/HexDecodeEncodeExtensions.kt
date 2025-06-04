package com.theguardian.coverdrop.core.utils

/**
 * Decodes a hexadecimal [String] as a [ByteArray]. Requires that [String.length] is divisible by 2.
 */
internal fun String.hexDecode(): ByteArray {
    check(length % 2 == 0)
    return chunked(2).map { it.toInt(16).toByte() }.toByteArray()
}

/**
 * Encodes a [ByteArray] as a lower-case hexadecimal [String].
 */
internal fun ByteArray.hexEncode(): String {
    val builder = java.lang.StringBuilder(size * 2)
    forEach { builder.append(String.format("%02x", it)) }
    return builder.toString()
}
