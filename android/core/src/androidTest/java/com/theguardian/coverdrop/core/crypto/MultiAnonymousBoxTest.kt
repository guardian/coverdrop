package com.theguardian.coverdrop.core.crypto

import androidx.test.platform.app.InstrumentationRegistry
import com.google.common.truth.Truth.assertThat
import com.theguardian.coverdrop.core.createLibSodium
import org.junit.Test
import kotlin.experimental.xor


class MultiAnonymousBoxTest {
    private val libSodium = createLibSodium()
    private val context = InstrumentationRegistry.getInstrumentation().context

    @Test
    fun testEncryptDecrypt_whenSameKeys_thenActualMatchesOriginal() {
        val recipient = EncryptionKeyPair.new(libSodium)
        val originalMessage = EncryptableVector.fromString("hello world")

        val box = MultiAnonymousBox.encrypt(
            libSodium = libSodium,
            recipientPks = listOf(recipient.publicEncryptionKey),
            data = originalMessage
        )

        val actualMessage = MultiAnonymousBox.decrypt(
            libSodium = libSodium,
            recipientPk = recipient.publicEncryptionKey,
            recipientSk = recipient.secretEncryptionKey,
            data = box,
            numRecipients = 1,
            constructor = ::EncryptableVector
        )

        assertThat(actualMessage).isEqualTo(originalMessage)
    }

    @Test(expected = IllegalStateException::class)
    fun testEncryptDecrypt_whenFlippingBitInCiphertext_thenDecryptFails() {
        val recipient = EncryptionKeyPair.new(libSodium)
        val originalMessage = EncryptableVector.fromString("hello world")

        val box = MultiAnonymousBox.encrypt(
            libSodium = libSodium,
            recipientPks = listOf(recipient.publicEncryptionKey),
            data = originalMessage
        )

        box.bytes[0] = box.bytes[0].xor(0x01)

        // this fails and throws
        MultiAnonymousBox.decrypt(
            libSodium = libSodium,
            recipientPk = recipient.publicEncryptionKey,
            recipientSk = recipient.secretEncryptionKey,
            data = box,
            numRecipients = 1,
            constructor = ::EncryptableVector
        )
    }

    @Test
    fun testEncryptDecrypt_whenMultipleRecipients_thenAllDecryptCorrectly() {
        val numRecipients = 2
        val recipients = List(numRecipients) { EncryptionKeyPair.new(libSodium) }
        val originalMessage = EncryptableVector.fromString("hello world")

        val box = MultiAnonymousBox.encrypt(
            libSodium = libSodium,
            recipientPks = recipients.map { it.publicEncryptionKey },
            data = originalMessage
        )

        for (recipient in recipients) {
            val actualMessage = MultiAnonymousBox.decrypt(
                libSodium = libSodium,
                recipientPk = recipient.publicEncryptionKey,
                recipientSk = recipient.secretEncryptionKey,
                data = box,
                numRecipients = numRecipients,
                constructor = ::EncryptableVector
            )
            assertThat(actualMessage).isEqualTo(originalMessage)
        }
    }

    @Test
    fun testDecrypt_whenUsingTestVector_thenMatchesExpected() {
        val testVectors = CryptoTestVectors(context, "multi_anonymous_box")
        val recipients = listOf(
            EncryptionKeyPair(
                publicEncryptionKey = testVectors.readPublicEncryptionKey("01_recipient_1_pk"),
                secretEncryptionKey = testVectors.readSecretEncryptionKey("02_recipient_1_sk"),
            ), EncryptionKeyPair(
                publicEncryptionKey = testVectors.readPublicEncryptionKey("03_recipient_2_pk"),
                secretEncryptionKey = testVectors.readSecretEncryptionKey("04_recipient_2_sk"),
            )
        )
        val message = testVectors.readEncryptableVector("05_message")
        val box =
            MultiAnonymousBox<EncryptableVector>(testVectors.readFile("06_multi_anonymous_box"))

        for (recipient in recipients) {
            val actualMessage = MultiAnonymousBox.decrypt(
                libSodium = libSodium,
                recipientPk = recipient.publicEncryptionKey,
                recipientSk = recipient.secretEncryptionKey,
                data = box,
                numRecipients = recipients.size,
                constructor = ::EncryptableVector
            )
            assertThat(actualMessage).isEqualTo(message)
        }
    }
}
