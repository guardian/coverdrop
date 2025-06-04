package com.theguardian.coverdrop.core.crypto

import androidx.test.platform.app.InstrumentationRegistry
import com.google.common.truth.Truth.assertThat
import com.theguardian.coverdrop.core.createLibSodium
import org.junit.Test
import kotlin.experimental.xor


class TwoPartyBoxTest {
    private val libSodium = createLibSodium()
    private val context = InstrumentationRegistry.getInstrumentation().context

    @Test
    fun testEncryptDecrypt_whenUsingMatchingKeys_thenResultMatchesExpected() {
        val sender = EncryptionKeyPair.new(libSodium)
        val recipient = EncryptionKeyPair.new(libSodium)

        val originalMessage = EncryptableVector.fromString("hello world")

        val box = TwoPartyBox.encrypt(
            libSodium = libSodium,
            recipientPk = recipient.publicEncryptionKey,
            senderSk = sender.secretEncryptionKey,
            data = originalMessage
        )

        val actualMessage = TwoPartyBox.decrypt(
            libSodium = libSodium,
            senderPk = sender.publicEncryptionKey,
            recipientSk = recipient.secretEncryptionKey,
            data = box,
            constructor = ::EncryptableVector
        )

        assertThat(actualMessage).isEqualTo(originalMessage)
    }

    @Test(expected = IllegalStateException::class)
    fun testEncryptDecrypt_whenFlippingBitInCiphertext_thenDecryptThrows() {
        val sender = EncryptionKeyPair.new(libSodium)
        val recipient = EncryptionKeyPair.new(libSodium)

        val originalMessage = EncryptableVector.fromString("hello world")

        val box = TwoPartyBox.encrypt(
            libSodium = libSodium,
            recipientPk = recipient.publicEncryptionKey,
            senderSk = sender.secretEncryptionKey,
            data = originalMessage
        )

        box.bytes[0] = box.bytes[0].xor(0x01)

        // this fails and throws
        TwoPartyBox.decrypt(
            libSodium = libSodium,
            senderPk = sender.publicEncryptionKey,
            recipientSk = recipient.secretEncryptionKey,
            data = box,
            constructor = ::EncryptableVector
        )
    }

    @Test
    fun testEncryptDecrypt_whenUsingTestVector_thenResultMatchesExpected() {
        val testVectors = CryptoTestVectors(context, "two_party_box")
        val sender = EncryptionKeyPair(
            publicEncryptionKey = testVectors.readPublicEncryptionKey("01_sender_pk"),
            secretEncryptionKey = testVectors.readSecretEncryptionKey("02_sender_sk"),
        )
        val recipient = EncryptionKeyPair(
            publicEncryptionKey = testVectors.readPublicEncryptionKey("03_recipient_pk"),
            secretEncryptionKey = testVectors.readSecretEncryptionKey("04_recipient_sk"),
        )
        val message = testVectors.readEncryptableVector("05_message")
        val box = TwoPartyBox<EncryptableVector>(testVectors.readFile("06_two_party_box"))

        val actualMessage = TwoPartyBox.decrypt(
            libSodium = libSodium,
            senderPk = sender.publicEncryptionKey,
            recipientSk = recipient.secretEncryptionKey,
            data = box,
            constructor = ::EncryptableVector
        )

        assertThat(actualMessage).isEqualTo(message)
    }
}
