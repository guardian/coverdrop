package com.theguardian.coverdrop.core.utils

import java.nio.BufferOverflowException
import java.nio.ByteBuffer


/**
 * Serializes a list by first writing its element count as a [Int] and then storing each of its
 * elements using [putLengthEncodedByteArray] for the output of the [serializeElement] function.
 *
 * The [maxSize] provides the maximum size of the returned [ByteArray]. If the serialization would
 * exceed this value a [java.nio.BufferOverflowException] is thrown.
 */
@kotlin.jvm.Throws(BufferOverflowException::class)
internal fun <E> List<E>.serializeOrThrow(
    maxSize: Int,
    serializeElement: (E) -> ByteArray,
): ByteArray {
    val buffer = ByteBuffer.allocate(maxSize)

    buffer.putInt(size)
    for (element in this) {
        buffer.putLengthEncodedByteArray(serializeElement(element))
    }

    return buffer.getWrittenBytes()
}

@Suppress("UsePropertyAccessSyntax")
internal fun <E> deserializeList(bytes: ByteArray, deserializeElement: (ByteArray) -> E): List<E> {
    val buffer = ByteBuffer.wrap(bytes)
    val size = buffer.getInt()
    return List(size) { deserializeElement(buffer.getLengthEncodedByteArray()) }
}
