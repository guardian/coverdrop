package com.theguardian.coverdrop.core.crypto

import com.goterl.lazysodium.SodiumAndroid
import com.goterl.lazysodium.interfaces.Box
import com.theguardian.coverdrop.core.utils.checkLibSodiumSuccess


internal class AnonymousBox<T>(internal val bytes: ByteArray) where T : Encryptable {
    companion object {
        fun <T> encrypt(
            libSodium: SodiumAndroid,
            recipientPk: PublicEncryptionKey,
            data: T,
        ): AnonymousBox<T> where T : Encryptable {
            val message = data.asUnencryptedBytes()
            val cipher = ByteArray(message.size + Box.SEALBYTES)

            val res = libSodium.crypto_box_seal(
                /* cipher = */ cipher,
                /* message = */ message,
                /* messageLen = */ message.size.toLong(),
                /* publicKey = */ recipientPk.bytes,
            )
            checkLibSodiumSuccess(res)

            return AnonymousBox(cipher)
        }

        fun <T> decrypt(
            libSodium: SodiumAndroid,
            recipientPk: PublicEncryptionKey,
            recipientSk: SecretEncryptionKey,
            data: AnonymousBox<T>,
            constructor: (ByteArray) -> T,
        ): T where T : Encryptable {
            val cipher = data.bytes
            require(cipher.size >= Box.SEALBYTES) { "bad data.bytes length" }

            val message = ByteArray(cipher.size - Box.SEALBYTES)

            val res = libSodium.crypto_box_seal_open(
                /* m = */ message,
                /* cipher = */ cipher,
                /* cipherLen = */ cipher.size.toLong(),
                /* publicKey = */ recipientPk.bytes,
                /* secretKey = */ recipientSk.bytes
            )
            checkLibSodiumSuccess(res)

            return constructor(message);
        }
    }
}
