package com.theguardian.coverdrop.core.utils

import java.nio.CharBuffer
import java.nio.charset.Charset
import java.util.Arrays
import kotlin.text.Charsets.UTF_8

/**
 * This encodes this [CharArray] into a [ByteArray] using the given [Charset]. This should return
 * the same results as [String.toByteArray]. All temporarily allocated copies are overwritten with
 * 0x00 bytes to reduce leakage against a memory adversary.
 *
 * "This" [CharArray] (the input) is not modified and needs manual erasure if the integrating code
 * continues to work only with the result.
 */
fun CharArray.toByteArray(charset: Charset = UTF_8): ByteArray {
    // wrapping in a `CharBuffer` does not allocate any new copy of the content
    val charBuffer = CharBuffer.wrap(this)

    // This runs the encoder and creates a copy in a `ByteBuffer`. This is mostly semantically
    // identical to `String.encodeToByteArray(...)`
    val byteBuffer = charset.encode(charBuffer)

    // the encode method is guaranteed to allocate an array-backed `ByteBuffer`
    check(byteBuffer.hasArray())
    val byteArray = byteBuffer.array()

    try {
        // copy content over (the underlying array's capacity might be larger)
        val toIndex = byteBuffer.limit()
        return byteArray.copyOfRange(0, toIndex)
    } finally {
        // erase the backing array of the `ByteBuffer`
        Arrays.fill(byteArray, 0x00)
    }
}
