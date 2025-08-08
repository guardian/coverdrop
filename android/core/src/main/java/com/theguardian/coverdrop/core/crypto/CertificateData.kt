package com.theguardian.coverdrop.core.crypto

import com.goterl.lazysodium.SodiumAndroid
import com.goterl.lazysodium.interfaces.Box
import com.theguardian.coverdrop.core.utils.checkLibSodiumSuccess
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.time.Instant


internal class SigningKeyCertificateData(private val byteArray: ByteArray) : Signable {
    override fun asBytes() = byteArray

    companion object {
        fun from(
            key: PublicSigningKey,
            notValidAfter: Instant,
        ): SigningKeyCertificateData {
            val buffer = ByteBuffer.allocate(Box.PUBLICKEYBYTES + Long.SIZE_BYTES)

            buffer.put(key.bytes)
            buffer.order(ByteOrder.BIG_ENDIAN)
            buffer.putLong(notValidAfter.epochSecond)
            check(buffer.remaining() == 0)

            return SigningKeyCertificateData(buffer.array())
        }
    }
}

internal class EncryptionKeyWithExpiryCertificateData(private val byteArray: ByteArray) : Signable {
    override fun asBytes() = byteArray

    companion object {
        fun from(
            key: PublicEncryptionKey,
            notValidAfter: Instant,
        ): EncryptionKeyWithExpiryCertificateData {
            val buffer = ByteBuffer.allocate(Box.PUBLICKEYBYTES + Long.SIZE_BYTES)

            buffer.put(key.bytes)
            buffer.order(ByteOrder.BIG_ENDIAN)
            buffer.putLong(notValidAfter.epochSecond)
            check(buffer.remaining() == 0)

            return EncryptionKeyWithExpiryCertificateData(buffer.array())
        }
    }
}

internal class DeadDropSignatureData(private val byteArray: ByteArray) : Signable {
    override fun asBytes() = byteArray

    companion object {
        fun from(
            libSodium: SodiumAndroid,
            data: ByteArray,
            createdAt: Instant
        ): DeadDropSignatureData {
            // See: `journalist_to_user_dead_drop_signature_data_v2.rs`
            val buffer = ByteBuffer.allocate(data.size + Long.SIZE_BYTES)
            buffer.put(data)
            buffer.order(ByteOrder.BIG_ENDIAN)
            buffer.putLong(createdAt.epochSecond)
            check(buffer.remaining() == 0)
            val array = buffer.array()

            val hashOutput = ByteArray(256 / 8)
            val res = libSodium.crypto_hash_sha256(hashOutput, array, array.size.toLong())
            checkLibSodiumSuccess(res)

            return DeadDropSignatureData(hashOutput)
        }
    }
}
