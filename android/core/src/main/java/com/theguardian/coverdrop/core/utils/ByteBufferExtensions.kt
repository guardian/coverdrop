package com.theguardian.coverdrop.core.utils

import java.nio.ByteBuffer

/**
 * Returns a [ByteArray] of the given [size] from this [ByteBuffer] (or throws a
 * `BufferUnderflowException` if not enough bytes are remaining).
 */
internal fun ByteBuffer.getByteArray(size: Int): ByteArray {
    val array = ByteArray(size)
    get(array)
    return array
}

/**
 * Returns the remaining content of the buffer as a [ByteArray].
 */
internal fun ByteBuffer.getRemainingAsByteArray(): ByteArray {
    val array = ByteArray(remaining())
    get(
        /* dst = */ array,
        /* offset = */ 0, // offset into the `dst` array
        /* length = */ remaining()
    )
    return array
}

/**
 * Returns a [ByteArray] of all bytes from the start of the buffer to the current position (excl)
 */
internal fun ByteBuffer.getWrittenBytes(): ByteArray {
    val pos = position()
    position(0)
    return getByteArray(pos)
}

const val LENGTH_ENCODING_OVERHEAD = Int.SIZE_BYTES

/**
 * Writes the byte array's length as a [Int] followed by the array contents.
 * See [ByteBuffer.getLengthEncodedByteArray].
 */
internal fun ByteBuffer.putLengthEncodedByteArray(bytes: ByteArray) {
    val length = bytes.size
    putInt(length)
    put(bytes)
}

/**
 * Reads a length encoded array from the buffer. See [ByteBuffer.putLengthEncodedByteArray].
 */
@Suppress("UsePropertyAccessSyntax")
internal fun ByteBuffer.getLengthEncodedByteArray(): ByteArray {
    val length = getInt()
    check(length >= 0)
    return getByteArray(length)
}

/**
 * Stores a boolean value as a single byte value.
 */
internal fun ByteBuffer.putBoolean(b: Boolean) {
    put(if (b) 0x01 else 0x00)
}

/**
 * Reads a boolean value from a single byte.
 */
internal fun ByteBuffer.getBoolean(): Boolean {
    return get() != 0.toByte()
}
