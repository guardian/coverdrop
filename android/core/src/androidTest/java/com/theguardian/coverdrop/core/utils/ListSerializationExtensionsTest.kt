package com.theguardian.coverdrop.core.utils

import com.google.common.truth.Correspondence
import com.google.common.truth.Truth.assertThat
import org.junit.Test
import java.nio.BufferOverflowException


class ListSerializationExtensionsTest {

    @Test
    fun testSerializeDeserialize_whenEmptyList_thenResultEmpty() {
        val original = emptyList<ByteArray>()

        val serialized = original.serializeOrThrow(maxSize = 1024, serializeElement = { it })
        assertThat(serialized.size).isLessThan(1024)

        val actual = deserializeList(bytes = serialized, deserializeElement = { it })
        assertThat(actual).isEqualTo(original)
    }

    @Test
    fun testSerializeDeserialize_whenElementsOfVariousSize_thenResultEqualToOriginal() {
        val original = listOf(
            "".encodeToByteArray(),
            "hello".encodeToByteArray(),
            "".encodeToByteArray(),
            "world".encodeToByteArray(),
        )

        val serialized = original.serializeOrThrow(maxSize = 1024, serializeElement = { it })
        assertThat(serialized.size).isLessThan(1024)

        val actual = deserializeList(bytes = serialized, deserializeElement = { it })
        assertThat(actual).comparingElementsUsing(ByteArrayContentEquals)
            .containsExactlyElementsIn(original)
    }

    @Test(expected = BufferOverflowException::class)
    fun testSerializeDeserialize_whenExceedsMaxSize_thenOverflowThrown() {
        val original = listOf(
            "hello".encodeToByteArray(),
        )

        // total length: 4 + (4 + 5) = 13 > 12
        original.serializeOrThrow(maxSize = 12, serializeElement = { it })
    }

    @Test
    fun testSerializeDeserialize_whenMatchesExactlyMaxSize_thenResultEqualToOriginal() {
        val original = listOf(
            "hello".encodeToByteArray(),
        )

        // total length: 4 + (4 + 5) = 13 <= 13
        val serialized = original.serializeOrThrow(maxSize = 13, serializeElement = { it })
        assertThat(serialized.size).isEqualTo(13)

        val actual = deserializeList(bytes = serialized, deserializeElement = { it })
        assertThat(actual).comparingElementsUsing(ByteArrayContentEquals)
            .containsExactlyElementsIn(original)
    }
}


val ByteArrayContentEquals: Correspondence<ByteArray, ByteArray> = Correspondence.from(
    { obj: ByteArray?, actual: ByteArray? -> obj.contentEquals(actual) },
    "is equivalent to"
)
