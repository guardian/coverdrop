package com.theguardian.coverdrop.core.models

import com.lambdapioneer.sloth.utils.secureRandomBytes
import com.theguardian.coverdrop.core.crypto.Encryptable
import com.theguardian.coverdrop.core.generated.AUTHENTICATED_MESSAGE_PADDING_LEN
import com.theguardian.coverdrop.core.generated.MESSAGE_PADDING_LEN
import com.theguardian.coverdrop.core.utils.getByteArray
import java.io.ByteArrayOutputStream
import java.nio.ByteBuffer
import java.nio.charset.StandardCharsets.UTF_8
import java.util.zip.GZIPInputStream
import java.util.zip.GZIPOutputStream

// The header for the padded string is a `short` (aka 2 bytes)
internal const val HEADER_SIZE = 2

sealed interface PaddedLength {
    val bytes: Int
    object U2J : PaddedLength { override val bytes = AUTHENTICATED_MESSAGE_PADDING_LEN }
    object J2U : PaddedLength { override val bytes = MESSAGE_PADDING_LEN }
}



class PaddedCompressedString<L : PaddedLength> private constructor (
    internal val bytes: ByteArray
) : Encryptable {
    companion object {
        fun userToJournalist(text: String) = fromString(text, PaddedLength.U2J)
        fun userToJournalist(bytes: ByteArray) = fromBytes(bytes, PaddedLength.U2J)

        fun journalistToUser(bytes: ByteArray) = fromBytes(bytes, PaddedLength.J2U)


        private fun <L: PaddedLength> fromString(text: String, length: L): PaddedCompressedString<L> {
            val outputStream = ByteArrayOutputStream();

            GZIPOutputStream(outputStream).bufferedWriter(UTF_8).use {
                it.write(text)
            }

            val compressedText = outputStream.toByteArray()
            require(compressedText.size < Short.MAX_VALUE)

            val lengthHeader = compressedText.size.toShort()

            val arr = secureRandomBytes(length.bytes)
            ByteBuffer.wrap(arr).apply {
                putShort(lengthHeader)
                // Throws buffer overflow is the message is too long for the allocated buffer
                put(compressedText)
            }

            return fromBytes(arr, length)
        }

        private fun <L: PaddedLength> fromBytes(bytes: ByteArray, length: L): PaddedCompressedString<L> {
            require(bytes.size == length.bytes)
            return PaddedCompressedString(bytes)
        }
    }

    fun toPayloadString(): String {
        val buf = ByteBuffer.wrap(bytes)

        val lengthHeader = buf.short

        val compressedString = buf.getByteArray(lengthHeader.toInt())
        val text = GZIPInputStream(compressedString.inputStream()).bufferedReader(UTF_8)
            .use { it.readText() }

        // The maximum compression ratio is ~1000:1. This is our (256 byte) messages would not
        // decode to output larger than 256 KiB. Nevertheless, we assume that everything with a
        // compression ratio larger than 100:1 is suspicious for natural text and we drop it.
        // See: https://github.com/guardian/coverdrop-internal/issues/112
        val decompressionRatio = text.length / compressedString.size;
        require(decompressionRatio < 100)

        return text
    }

    fun totalLength(): Int {
        return bytes.size
    }

    fun paddingLength(): Int {
        return bytes.size - HEADER_SIZE - compressedDataLength()
    }

    fun fillLevel(): Float {
        val maxCompressedData = bytes.size - HEADER_SIZE
        return compressedDataLength().toFloat() / maxCompressedData.toFloat()
    }

    private fun compressedDataLength(): Int {
        val buffer = ByteBuffer.wrap(bytes)
        return buffer.short.toInt()
    }

    override fun asUnencryptedBytes(): ByteArray {
        return bytes;
    }
}

internal typealias U2JPaddedCompressedString = PaddedCompressedString<PaddedLength.U2J>
internal typealias J2UPaddedCompressedString = PaddedCompressedString<PaddedLength.J2U>