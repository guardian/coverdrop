package com.theguardian.coverdrop.core.crypto

import com.google.common.truth.Truth.assertThat
import com.theguardian.coverdrop.core.api.models.VerifiedSignedEncryptionKey
import com.theguardian.coverdrop.core.createLibSodium
import com.theguardian.coverdrop.core.generated.USER_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN
import com.theguardian.coverdrop.core.models.PaddedCompressedString
import org.junit.Test
import java.time.Instant

@Suppress("UsePropertyAccessSyntax")
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
            userMsgKeyPair.publicEncryptionKey,
            PaddedCompressedString.fromString("Hello David!"),
            "C0FF33EE",
        )
        assertThat(messageBytes).hasLength(USER_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN)
    }
}
