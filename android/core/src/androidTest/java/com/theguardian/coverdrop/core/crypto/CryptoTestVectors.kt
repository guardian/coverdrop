package com.theguardian.coverdrop.core.crypto

import android.content.Context
import com.theguardian.coverdrop.testutils.TestVectors
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.time.Instant
import java.time.ZonedDateTime

internal class CryptoTestVectors(context: Context, basePath: String) :
    TestVectors(context, basePath) {

    fun readEncryptableVector(path: String) = EncryptableVector(readFile(path))

    fun readPublicEncryptionKey(path: String) = PublicEncryptionKey(readFile(path))

    fun readSecretEncryptionKey(path: String) = SecretEncryptionKey(readFile(path))

    fun readSignableVector(path: String) = SignableVector(readFile(path))

    fun readPublicSigningKey(path: String) = PublicSigningKey(readFile(path))

    fun readSecretSigningKey(path: String) = SecretSigningKey(readFile(path))

    fun readInstant(path: String): Instant {
        // we need the ZonedDateTime parser here to handle the +00:00 formatting of the vector
        val zonedDateTime = ZonedDateTime.parse(readFile(path).decodeToString())
        return zonedDateTime.toInstant()
    }

    fun readTimestampBigEndian(path: String): Any {
        val buffer = ByteBuffer.wrap(readFile(path))
        buffer.order(ByteOrder.BIG_ENDIAN)

        @Suppress("UsePropertyAccessSyntax")
        return buffer.getLong()
    }
}
