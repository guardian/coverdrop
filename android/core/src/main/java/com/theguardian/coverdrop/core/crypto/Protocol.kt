package com.theguardian.coverdrop.core.crypto

import com.goterl.lazysodium.SodiumAndroid
import com.theguardian.coverdrop.core.api.models.CoverNodeId
import com.theguardian.coverdrop.core.api.models.VerifiedSignedEncryptionKey
import com.theguardian.coverdrop.core.generated.COVERNODE_WRAPPING_KEY_COUNT
import com.theguardian.coverdrop.core.generated.MESSAGE_PADDING_LEN
import com.theguardian.coverdrop.core.generated.RECIPIENT_TAG_BYTES_U2J_COVER
import com.theguardian.coverdrop.core.generated.RECIPIENT_TAG_LEN
import com.theguardian.coverdrop.core.generated.USER_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN
import com.theguardian.coverdrop.core.generated.USER_TO_COVERNODE_MESSAGE_LEN
import com.theguardian.coverdrop.core.generated.USER_TO_JOURNALIST_ENCRYPTED_MESSAGE_LEN
import com.theguardian.coverdrop.core.generated.USER_TO_JOURNALIST_MESSAGE_LEN
import com.theguardian.coverdrop.core.models.JournalistTag
import com.theguardian.coverdrop.core.models.PaddedCompressedString
import com.theguardian.coverdrop.core.utils.hexDecode
import com.theguardian.coverdrop.core.utils.nextByteArray
import java.security.SecureRandom


/**
 * Similar to the `common::protocol` crate within the Rust project, this class implements the
 * high-level operations for creating and encrypting user messages.
 */
internal class Protocol(private val libSodium: SodiumAndroid) {

    /**
     * Creates a new message for cover traffic. Since it is anonymously sent to just the
     * CoverNode, this method only requires its message key.
     */
    fun createCoverMessageToCoverNode(coverNodesToMostRecentKey: Map<CoverNodeId, VerifiedSignedEncryptionKey>): ByteArray {
        val random = SecureRandom()

        // create placeholder string instead of inner message
        val innerEncryptedPlaceholder =
            random.nextByteArray(USER_TO_JOURNALIST_ENCRYPTED_MESSAGE_LEN)
        check(innerEncryptedPlaceholder.size == USER_TO_JOURNALIST_ENCRYPTED_MESSAGE_LEN)

        // build payload of the outer message (to be read by the CoverNode after decryption)
        val payloadForOuter = RECIPIENT_TAG_BYTES_U2J_COVER + innerEncryptedPlaceholder
        check(payloadForOuter.size == USER_TO_COVERNODE_MESSAGE_LEN)

        // pick the coverNode keys to encrypt to
        val coverNodeKeys = selectCoverNodeKeys(coverNodesToMostRecentKey)

        // encrypt outer message to CoverNode
        val outerEncryptedMessage = MultiAnonymousBox.encrypt(
            libSodium = libSodium,
            recipientPks = coverNodeKeys,
            data = EncryptableVector(payloadForOuter)
        )
        val outerEncryptedMessageBytes = outerEncryptedMessage.bytes
        check(outerEncryptedMessageBytes.size == USER_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN)

        return outerEncryptedMessageBytes
    }

    /**
     * Encrypts the given [paddedMessage] for the given [journalistTag] and [journalistMsgKey].
     */
    fun encryptUserToJournalistMessageViaCoverNode(
        coverNodesToMostRecentKey: Map<CoverNodeId, VerifiedSignedEncryptionKey>,
        journalistMsgKey: VerifiedSignedEncryptionKey,
        userMsgKey: PublicEncryptionKey,
        paddedMessage: PaddedCompressedString,
        journalistTag: JournalistTag,
    ): ByteArray {
        require(paddedMessage.bytes.size == MESSAGE_PADDING_LEN)

        val journalistTagBytes = journalistTag.hexDecode()
        require(journalistTagBytes.size == RECIPIENT_TAG_LEN)

        // build payload of inner message (to be read by the journalist after decryption)
        val reservedByte = ByteArray(1)
        val payloadForInnerMessage = userMsgKey.bytes + reservedByte + paddedMessage.bytes
        check(payloadForInnerMessage.size == USER_TO_JOURNALIST_MESSAGE_LEN)

        // encrypt inner message to journalist
        val innerEncryptedMessage = AnonymousBox.encrypt(
            libSodium = libSodium,
            recipientPk = journalistMsgKey.pk,
            data = EncryptableVector(payloadForInnerMessage)
        )
        check(innerEncryptedMessage.bytes.size == USER_TO_JOURNALIST_ENCRYPTED_MESSAGE_LEN)

        // build payload of the outer message (to be read by the CoverNode after decryption)
        val payloadForOuter = journalistTagBytes + innerEncryptedMessage.bytes
        check(payloadForOuter.size == USER_TO_COVERNODE_MESSAGE_LEN)

        // pick the coverNode keys to encrypt to
        val coverNodeKeys = selectCoverNodeKeys(coverNodesToMostRecentKey)

        // encrypt outer message to CoverNodes
        val outerEncryptedMessage = MultiAnonymousBox.encrypt(
            libSodium = libSodium,
            recipientPks = coverNodeKeys,
            data = EncryptableVector(payloadForOuter)
        )
        val outerEncryptedMessageBytes = outerEncryptedMessage.bytes
        check(outerEncryptedMessageBytes.size == USER_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN)

        return outerEncryptedMessageBytes
    }

    /**
     * Selects the coverNode encryption keys (exactly [COVERNODE_WRAPPING_KEY_COUNT] many) to
     * for encrypting the outer message. If the given keys are more than the output list size,
     * then the first ones are chosen. Otherwise, the first ones are repeated.
     */
    private fun selectCoverNodeKeys(coverNodesToMostRecentKey: Map<CoverNodeId, VerifiedSignedEncryptionKey>): List<PublicEncryptionKey> {
        val allCoverNodeKeys = coverNodesToMostRecentKey.values.toList()
        return List(COVERNODE_WRAPPING_KEY_COUNT) { allCoverNodeKeys[it % allCoverNodeKeys.size].pk }
    }

}
