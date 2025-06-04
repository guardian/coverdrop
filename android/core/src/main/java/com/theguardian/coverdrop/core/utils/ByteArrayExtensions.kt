package com.theguardian.coverdrop.core.utils

/**
 * Returns a padded copy of length [len] of this [ByteArray]. The padding bytes have value [filler].
 *
 * @throws IllegalArgumentException if the target [len] is smaller than the [ByteArray.size].
 */
internal fun ByteArray.padTo(len: Int, filler: Byte = 0x00): ByteArray {
    require(size <= len)
    val padding = ByteArray(len - size) { filler }
    return this + padding
}

/**
 * Chunks this [ByteArray] into a list of [ByteArray]s of size [chunkSize].
 *
 * @throws IllegalArgumentException if the [chunkSize] is not a positive integer or if the
 * [ByteArray] does not divide evenly into chunks.
 */
internal fun ByteArray.chunked(chunkSize: Int): List<ByteArray> {
    require(chunkSize > 0)
    require(size % chunkSize == 0)
    return List(size / chunkSize) { i ->
        val offset = i * chunkSize
        this.sliceArray(offset until offset + chunkSize)
    }
}

/**
 * Splits the array at `offset` into a pair of arrays such that the first array contains the
 * elements before (and excluding) `offset` and the second array contains the elements
 * (including and) after `offset`.
 */
internal fun ByteArray.splitAt(offset: Int): Pair<ByteArray, ByteArray> {
    require(offset >= 0)
    require(offset <= size)
    return Pair(
        sliceArray(0 until offset),
        sliceArray(offset until size)
    )
}
