package com.theguardian.coverdrop.core.crypto

import com.goterl.lazysodium.SodiumAndroid
import com.goterl.lazysodium.interfaces.Box
import com.goterl.lazysodium.interfaces.SecretBox
import com.theguardian.coverdrop.core.utils.checkLibSodiumSuccess
import com.theguardian.coverdrop.core.utils.chunked
import com.theguardian.coverdrop.core.utils.splitAt
import java.nio.ByteBuffer

const val WRAPPED_KEY_SIZE = SecretBox.KEYBYTES + Box.SEALBYTES

internal class MultiAnonymousBox<T>(internal val bytes: ByteArray) where T : Encryptable {
    companion object {
        fun <T> encrypt(
            libSodium: SodiumAndroid,
            recipientPks: List<PublicEncryptionKey>,
            data: T,
        ): MultiAnonymousBox<T> where T : Encryptable {
            val message = data.asUnencryptedBytes()

            val key = ByteArray(SecretBox.KEYBYTES)
            libSodium.crypto_secretbox_keygen(key)
            require(!key.all { it == 0x00.toByte() }) // keygen has no return value, so we do a paranoid check here

            val ciphertext = encryptWithSecretBox(
                libSodium = libSodium,
                key = key,
                nonce = ByteArray(SecretBox.NONCEBYTES), // since we always use fresh keys for each message, we can choose a constant nonce
                message = message,
            )

            val keyAsEncryptableVector = EncryptableVector(key)
            val wrappedKeys = recipientPks.map { recipientPk ->
                AnonymousBox.encrypt(
                    libSodium = libSodium,
                    recipientPk = recipientPk,
                    data = keyAsEncryptableVector
                )
            }

            val output = ByteBuffer.allocate(wrappedKeys.size * WRAPPED_KEY_SIZE + ciphertext.size)
            wrappedKeys.forEach { output.put(it.bytes) }
            output.put(ciphertext)
            check(output.remaining() == 0)

            return MultiAnonymousBox(output.array())
        }

        fun <T> decrypt(
            libSodium: SodiumAndroid,
            recipientPk: PublicEncryptionKey,
            recipientSk: SecretEncryptionKey,
            data: MultiAnonymousBox<T>,
            numRecipients: Int,
            constructor: (ByteArray) -> T,
        ): T where T : Encryptable {
            val bytes = data.bytes
            require(bytes.size >= numRecipients * WRAPPED_KEY_SIZE + SecretBox.MACBYTES) { "bad data.bytes length" }

            val (wrappedKeys, ciphertext) = bytes.splitAt(numRecipients * WRAPPED_KEY_SIZE)

            val key = findKey(libSodium, wrappedKeys, recipientPk, recipientSk)
            checkNotNull(key) { "failed to decrypt message: no matching key found" }

            val message = ByteArray(ciphertext.size - SecretBox.MACBYTES)

            val res = libSodium.crypto_secretbox_open_easy(
                /* message = */ message,
                /* cipherText = */
                ciphertext,
                /* cipherTextLen = */
                ciphertext.size.toLong(),
                /* nonce = */
                ByteArray(SecretBox.NONCEBYTES), // since we always use fresh keys for each message, we can choose a constant nonce
                /* key = */
                key.asUnencryptedBytes()
            )
            checkLibSodiumSuccess(res)

            return constructor(message);
        }

        private fun encryptWithSecretBox(
            libSodium: SodiumAndroid,
            key: ByteArray,
            nonce: ByteArray,
            message: ByteArray,
        ): ByteArray {
            val ciphertext = ByteArray(message.size + SecretBox.MACBYTES)
            val res = libSodium.crypto_secretbox_easy(
                /* cipherText = */ ciphertext,
                /* message = */ message,
                /* messageLen = */ message.size.toLong(),
                /* nonce = */ nonce,
                /* key = */ key
            )
            checkLibSodiumSuccess(res)
            return ciphertext
        }

        private fun findKey(
            libSodium: SodiumAndroid,
            wrappedKeys: ByteArray,
            recipientPk: PublicEncryptionKey,
            recipientSk: SecretEncryptionKey,
        ) = wrappedKeys
            .chunked(WRAPPED_KEY_SIZE)
            .firstNotNullOfOrNull { wrappedKey ->
                try {
                    AnonymousBox.decrypt(
                        libSodium = libSodium,
                        recipientPk = recipientPk,
                        recipientSk = recipientSk,
                        data = AnonymousBox(wrappedKey),
                        constructor = ::EncryptableVector
                    )
                } catch (e: IllegalStateException) {
                    null
                }
            }
    }
}
