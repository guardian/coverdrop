package com.theguardian.coverdrop.core.crypto

import com.goterl.lazysodium.SodiumAndroid
import com.goterl.lazysodium.interfaces.Box
import com.theguardian.coverdrop.core.utils.checkLibSodiumSuccess


internal class TwoPartyBox<T>(internal val bytes: ByteArray) where T : Encryptable {
    companion object {
        fun <T> encrypt(
            libSodium: SodiumAndroid,
            recipientPk: PublicEncryptionKey,
            senderSk: SecretEncryptionKey,
            data: T,
        ): TwoPartyBox<T> where T : Encryptable {
            val message = data.asUnencryptedBytes()
            val ciphertext = ByteArray(message.size + Box.MACBYTES)

            val nonce = ByteArray(Box.NONCEBYTES)
            libSodium.randombytes_buf(/* buffer = */ nonce, /* size = */ nonce.size);

            val res = libSodium.crypto_box_easy(
                /* cipherText = */ ciphertext,
                /* message = */ message,
                /* messageLen = */ message.size.toLong(),
                /* nonce = */ nonce,
                /* publicKey = */ recipientPk.bytes,
                /* secretKey = */ senderSk.bytes
            )
            checkLibSodiumSuccess(res)

            val twoPartyBoxBytes = ciphertext + nonce
            return TwoPartyBox(twoPartyBoxBytes)
        }

        fun <T> decrypt(
            libSodium: SodiumAndroid,
            senderPk: PublicEncryptionKey,
            recipientSk: SecretEncryptionKey,
            data: TwoPartyBox<T>,
            constructor: (ByteArray) -> T,
        ): T where T : Encryptable {
            val twoPartyBoxBytes = data.bytes
            require(twoPartyBoxBytes.size >= Box.MACBYTES + Box.NONCEBYTES) { "bad twoPartyBoxBytes length" }

            val offsetNonceBytes = twoPartyBoxBytes.size - Box.NONCEBYTES
            val ciphertext = twoPartyBoxBytes
                .slice(0 until offsetNonceBytes)
                .toByteArray()
            val nonce = twoPartyBoxBytes
                .slice(offsetNonceBytes until twoPartyBoxBytes.size)
                .toByteArray()
            val message = ByteArray(ciphertext.size - Box.MACBYTES)

            val res = libSodium.crypto_box_open_easy(
                /* message = */ message,
                /* cipherText = */ ciphertext,
                /* cipherTextLen = */ ciphertext.size.toLong(),
                /* nonce = */ nonce,
                /* publicKey = */ senderPk.bytes,
                /* secretKey = */ recipientSk.bytes
            )
            checkLibSodiumSuccess(res)

            return constructor(message);
        }
    }
}
