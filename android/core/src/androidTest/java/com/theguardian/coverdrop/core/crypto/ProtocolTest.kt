package com.theguardian.coverdrop.core.crypto

import com.google.common.truth.Truth.assertThat
import com.theguardian.coverdrop.core.api.models.VerifiedSignedEncryptionKey
import com.theguardian.coverdrop.core.createLibSodium
import com.theguardian.coverdrop.core.generated.AUTHENTICATED_MESSAGE_RESERVED_BYTE_VALUE
import com.theguardian.coverdrop.core.generated.RECIPIENT_TAG_LEN
import com.theguardian.coverdrop.core.generated.USER_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN
import com.theguardian.coverdrop.core.models.PaddedCompressedString
import com.theguardian.coverdrop.core.models.U2JPaddedCompressedString
import com.theguardian.coverdrop.core.utils.hexDecode
import org.junit.Test
import java.time.Instant

class ProtocolTest {

    private val libSodium = createLibSodium()
    private val instance = Protocol(libSodium)

    private val coverNodeMsgKeyPair = EncryptionKeyPair.new(libSodium)
    private val coverNodesToMostRecentKey = mapOf(
        "covernode_001" to VerifiedSignedEncryptionKey(
            pk = coverNodeMsgKeyPair.publicEncryptionKey,
            notValidAfter = Instant.now(),
        )
    )

    private val journalistMsgKeyPair = EncryptionKeyPair.new(libSodium)
    private val journalistMsgKey = VerifiedSignedEncryptionKey(
        pk = journalistMsgKeyPair.publicEncryptionKey,
        notValidAfter = Instant.now(),
    )

    private val userMsgKeyPair = EncryptionKeyPair.new(libSodium)

    @Test
    fun testCreateCoverMessageToCoverNode_whenGivenValidKey_thenOutputLenAsExpected() {
        val messageBytes = instance.createCoverMessageToCoverNode(coverNodesToMostRecentKey)
        assertThat(messageBytes).hasLength(USER_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN)
    }

    @Test
    fun testEncryptMessageToJournalist_whenGivenValidKey_thenOutputLenAsExpected() {
        val messageBytes = instance.encryptUserToJournalistMessageViaCoverNode(
            coverNodesToMostRecentKey,
            journalistMsgKey,
            userMsgKeyPair,
            PaddedCompressedString.userToJournalist("Hello David!"),
            "C0FF33EE",
        )
        assertThat(messageBytes).hasLength(USER_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN)
    }

    @Test
    fun testEncryptMessageToJournalist_whenGivenValidKey_thenMessageRoundTripsExpected() {
        val message = "Hello David!"

        val journalistTag = "C0FF33EE"

        // Create encrypted U2C message
        val messageBytes = instance.encryptUserToJournalistMessageViaCoverNode(
            coverNodesToMostRecentKey,
            journalistMsgKey,
            userMsgKeyPair,
            PaddedCompressedString.userToJournalist(message),
            journalistTag
        )
        assertThat(messageBytes).hasLength(USER_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN)

        // Decrypt U2C message [MultiAnonymousBox]
        val outerBox = MultiAnonymousBox<EncryptableVector>(messageBytes)

        val openedOuterBox = MultiAnonymousBox.decrypt(
            libSodium,
            coverNodeMsgKeyPair.publicEncryptionKey,
            coverNodeMsgKeyPair.secretEncryptionKey,
            outerBox,
            2,
            ::EncryptableVector
        ).asUnencryptedBytes()

        val recipientTag = openedOuterBox.take(RECIPIENT_TAG_LEN)
        assertThat(recipientTag).isEqualTo(journalistTag.hexDecode().toList())

        // Decrypt U2J message [AnonymousBox]
        val anonymousBox = AnonymousBox<EncryptableVector>(openedOuterBox.sliceArray(RECIPIENT_TAG_LEN..openedOuterBox.size - 1))

        val openBoxBytes = AnonymousBox.decrypt(
             libSodium,
             journalistMsgKeyPair.publicEncryptionKey,
             journalistMsgKeyPair.secretEncryptionKey,
            anonymousBox,
             ::EncryptableVector
         ).asUnencryptedBytes()

        val replyKey = openBoxBytes.take(32)

        assertThat(replyKey).isEqualTo(userMsgKeyPair.publicEncryptionKey.bytes.toList())

        val reservedByte = openBoxBytes.drop(32).take(1)[0]

        assertThat(reservedByte).isEqualTo(AUTHENTICATED_MESSAGE_RESERVED_BYTE_VALUE.toByte())

        // Decrypt U2J message payload [TwoPartyBox]
        val twoPartyBox = TwoPartyBox<EncryptableVector>(openBoxBytes.drop(33).toByteArray())

        val openInnerBoxBytes = TwoPartyBox.decrypt(
            libSodium,
            userMsgKeyPair.publicEncryptionKey,
            journalistMsgKeyPair.secretEncryptionKey,
            twoPartyBox,
            ::EncryptableVector
        ).asUnencryptedBytes()

        val decryptedMessage = PaddedCompressedString.userToJournalist(openInnerBoxBytes).toPayloadString()

        // Assert decrypted messages matches original
        assertThat(decryptedMessage).isEqualTo(message)

    }
}
