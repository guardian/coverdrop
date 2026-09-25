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

/**
 * The role of an identity key. The [entityName] is bound into the [IdKeyCertificateData] so that
 * e.g. a journalist identity key cannot be confused with a CoverNode identity key of the same id.
 *
 * See: `entity_name` in `roles.rs`
 */
internal enum class IdKeyRole(val entityName: String) {
    JOURNALIST_ID("journalist_id"),
    COVERNODE_ID("covernode_id"),
}

internal class IdKeyCertificateData(private val byteArray: ByteArray) : Signable {
    override fun asBytes() = byteArray

    companion object {
        private val ID_KEY_CERT_DATA_TAG = "ID_KEY_CERT_DATA".toByteArray(Charsets.US_ASCII)

        /**
         * Layout (length-prefixed for variable fields). See: `id_key_certificate_data.rs`
         * ```
         * [
         *     "ID_KEY_CERT_DATA" (16B) |
         *     key (32B) |
         *     not_valid_after_be_i64 (8B) |
         *     role_len_be_u32 (4B) | role_name |
         *     id_len_be_u32 (4B) | id_utf8
         * ]
         * ```
         */
        fun from(
            key: PublicSigningKey,
            notValidAfter: Instant,
            role: IdKeyRole,
            identity: String,
        ): IdKeyCertificateData {
            val roleBytes = role.entityName.toByteArray(Charsets.UTF_8)
            val identityBytes = identity.toByteArray(Charsets.UTF_8)
            val buffer = ByteBuffer.allocate(
                ID_KEY_CERT_DATA_TAG.size +
                        ED25519_PUBLIC_KEY_BYTES +
                        Long.SIZE_BYTES +
                        Int.SIZE_BYTES + roleBytes.size +
                        Int.SIZE_BYTES + identityBytes.size
            )

            buffer.order(ByteOrder.BIG_ENDIAN)
            buffer.put(ID_KEY_CERT_DATA_TAG)
            buffer.put(key.bytes)
            buffer.putLong(notValidAfter.epochSecond)
            buffer.putInt(roleBytes.size)
            buffer.put(roleBytes)
            buffer.putInt(identityBytes.size)
            buffer.put(identityBytes)
            check(buffer.remaining() == 0)

            return IdKeyCertificateData(buffer.array())
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
