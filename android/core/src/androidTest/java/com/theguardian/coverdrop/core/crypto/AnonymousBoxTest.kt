package com.theguardian.coverdrop.core.crypto

import androidx.test.platform.app.InstrumentationRegistry
import com.google.common.truth.Truth.assertThat
import com.theguardian.coverdrop.core.createLibSodium
import org.junit.Test
import kotlin.experimental.xor


class AnonymousBoxTest {
    private val libSodium = createLibSodium()
    private val context = InstrumentationRegistry.getInstrumentation().context

    @Test
    fun testEncryptDecrypt_whenSameKeys_thenActualMatchesOriginal() {
        val recipient = EncryptionKeyPair.new(libSodium)
        val originalMessage = EncryptableVector.fromString("hello world")

        val box = AnonymousBox.encrypt(
            libSodium = libSodium,
            recipientPk = recipient.publicEncryptionKey,
            data = originalMessage
        )

        val actualMessage = AnonymousBox.decrypt(
            libSodium = libSodium,
            recipientPk = recipient.publicEncryptionKey,
            recipientSk = recipient.secretEncryptionKey,
            data = box,
            constructor = ::EncryptableVector
        )

        assertThat(actualMessage).isEqualTo(originalMessage)
    }

    @Test(expected = IllegalStateException::class)
    fun testEncryptDecrypt_whenFlippingBitInCiphertext_thenDecryptFails() {
        val recipient = EncryptionKeyPair.new(libSodium)
        val originalMessage = EncryptableVector.fromString("hello world")

        val box = AnonymousBox.encrypt(
            libSodium = libSodium,
            recipientPk = recipient.publicEncryptionKey,
            data = originalMessage
        )

        box.bytes[0] = box.bytes[0].xor(0x01)

        // this fails and throws
        AnonymousBox.decrypt(
            libSodium = libSodium,
            recipientPk = recipient.publicEncryptionKey,
            recipientSk = recipient.secretEncryptionKey,
            data = box,
            constructor = ::EncryptableVector
        )
    }

    @Test
    fun testDecrypt_whenUsingTestVector_thenMatchesExpected() {
        val testVectors = CryptoTestVectors(context, "anonymous_box")
        val recipient = EncryptionKeyPair(
            publicEncryptionKey = testVectors.readPublicEncryptionKey("01_recipient_pk"),
            secretEncryptionKey = testVectors.readSecretEncryptionKey("02_recipient_sk"),
        )
        val message = testVectors.readEncryptableVector("03_message")
        val box = AnonymousBox<EncryptableVector>(testVectors.readFile("04_anonymous_box"))

        val actualMessage = AnonymousBox.decrypt(
            libSodium = libSodium,
            recipientPk = recipient.publicEncryptionKey,
            recipientSk = recipient.secretEncryptionKey,
            data = box,
            constructor = ::EncryptableVector
        )

        assertThat(actualMessage).isEqualTo(message)
    }
}
