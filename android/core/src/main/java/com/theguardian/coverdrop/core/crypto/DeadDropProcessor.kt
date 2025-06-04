package com.theguardian.coverdrop.core.crypto

import androidx.annotation.VisibleForTesting
import com.goterl.lazysodium.SodiumAndroid
import com.theguardian.coverdrop.core.api.models.VerifiedDeadDrops
import com.theguardian.coverdrop.core.api.models.VerifiedJournalistsKeyHierarchy
import com.theguardian.coverdrop.core.api.models.VerifiedSignedEncryptionKey
import com.theguardian.coverdrop.core.api.models.allJournalistToMessagingKeys
import com.theguardian.coverdrop.core.models.JournalistId
import com.theguardian.coverdrop.core.persistence.StoredMessage
import com.theguardian.coverdrop.core.persistence.StoredMessageThread
import com.theguardian.coverdrop.core.persistence.StoredMessageThreads
import java.time.Instant

/**
 * The [DeadDropProcessor] can decrypt incoming [VerifiedDeadDrops] and merge them with the
 * existing [StoredMessageThreads].
 */
internal class DeadDropProcessor(private val libSodium: SodiumAndroid) {

    /**
     * Combined execution of [decryptIncomingDeadDrops] and [mergeExistingThreadsWithNewMessages].
     * It first identifies all messages that can be decrypted from the newest [deadDrops] and then
     * merges them with the [existingMessageThreads]. The newly merged state is returned as a [List]
     * of new [StoredMessageThread] objects.
     *
     * Implementation note: designed as a pure function to facilitate testing.
     */
    fun decryptAndMerge(
        existingMessageThreads: StoredMessageThreads,
        deadDrops: VerifiedDeadDrops,
        journalistsKeyHierarchies: List<VerifiedJournalistsKeyHierarchy>,
        userKeyPair: EncryptionKeyPair,
    ): StoredMessageThreads {
        val knownJournalistIds = existingMessageThreads.map { it.recipientId }.toSet()
        val newMessages = decryptIncomingDeadDrops(
            deadDrops = deadDrops,
            journalistsKeyHierarchies = journalistsKeyHierarchies,
            knownJournalistIds = knownJournalistIds,
            userKeyPair = userKeyPair,
        )
        return mergeExistingThreadsWithNewMessages(existingMessageThreads, newMessages)
    }

    /**
     * Appends the [newMessages] to the [existingMessageThreads] by grouping based on common
     * [JournalistId]. Any new messages where there is no existing thread for that id, are ignored.
     *
     * Implementation note: designed as a pure function to facilitate testing.
     */
    @VisibleForTesting
    internal fun mergeExistingThreadsWithNewMessages(
        existingMessageThreads: StoredMessageThreads,
        newMessages: List<DecryptedDeadDropMessage>,
    ): StoredMessageThreads {
        // group existing messages by journalist identifier: building the new list and checking for
        // existing messages
        val mapIdToExisting = HashMap<JournalistId, MutableList<StoredMessage>>()
        val mapIdToExistingSet = HashMap<JournalistId, HashSet<StoredMessage>>()
        for (thread in existingMessageThreads) {
            mapIdToExisting[thread.recipientId] = thread.messages.toMutableList()
            mapIdToExistingSet[thread.recipientId] = thread.messages.toHashSet()
        }

        // add new messages to the respective threads (or create a new one)
        for (message in newMessages) {
            val newMessage = when (message) {
                is DecryptedDeadDropMessage.Text -> StoredMessage.remote(
                    message.timestamp,
                    message.message
                )

                is DecryptedDeadDropMessage.Handover -> StoredMessage.remoteHandover(
                    message.timestamp,
                    message.handoverTo
                )

                is DecryptedDeadDropMessage.Unknown -> StoredMessage.remoteUnknown(message.timestamp)
            }

            if (mapIdToExisting.containsKey(message.remoteId)) {
                if (!mapIdToExistingSet[message.remoteId]!!.add(newMessage)) {
                    // `add` returns `false` if the element was already contained in the set which
                    // means that this is a duplicate message, we can ignore it
                    continue
                }
                mapIdToExisting[message.remoteId]!!.add(newMessage)
                mapIdToExistingSet[message.remoteId]!!.add(newMessage)
            } else {
                // this would generally be a rare occasion: it would require that the user writes
                // to a journalist, then their entire conversation gets eventually truncated when
                // we serialize the mailbox, and then they receive a message
                mapIdToExisting[message.remoteId] = mutableListOf(newMessage)
                mapIdToExistingSet[message.remoteId] = hashSetOf(newMessage)
            }
        }

        // create new `StoredMessageThreads`
        return mapIdToExisting.map { entry ->
            StoredMessageThread(recipientId = entry.key, messages = entry.value.toList())
        }
    }

    /**
     * Takes the [deadDrops] and tries decrypting every message in all contained dead-drops
     * using [journalistsKeyHierarchies]. However, we only consider keys from journalists that
     * are within the [knownJournalistIds] list.
     *
     * The runtime of this method is linear in both the total number of messages (number of dead-
     * drops TIMES messages per dead-drop) and the total number of journalist message keys (number
     * of journalist ids TIMES keys per journalist).
     *
     * All successful attempts are included in the returned list of [DecryptedDeadDropMessage]s.
     *
     * Implementation note: designed as a pure function to facilitate testing.
     */
    @VisibleForTesting
    internal fun decryptIncomingDeadDrops(
        deadDrops: VerifiedDeadDrops,
        journalistsKeyHierarchies: List<VerifiedJournalistsKeyHierarchy>,
        knownJournalistIds: Set<JournalistId>,
        userKeyPair: EncryptionKeyPair,
    ): List<DecryptedDeadDropMessage> {
        val decryptedMessages = mutableListOf<DecryptedDeadDropMessage>()
        val journalistsToMessagingKeys = journalistsKeyHierarchies
            .allJournalistToMessagingKeys()
            .filterKeys { knownJournalistIds.contains(it) }

        // for all dead-drops
        for (deadDrop in deadDrops) {

            // and for all messages within those
            for (message in deadDrop.messages) {

                // try decrypting for all journalists
                val maybeDecryptedDeadDropMessage = tryDecryptForJournalistsOrNull(
                    journalistsToMessagingKeys,
                    userKeyPair,
                    message,
                    timestamp = deadDrop.createdAt,
                )

                // if it was successful decrypted by any key, add to the result list
                if (maybeDecryptedDeadDropMessage != null) {
                    decryptedMessages.add(maybeDecryptedDeadDropMessage)
                }
            } // all messages
        } // all dead-drops

        return decryptedMessages
    }

    private fun tryDecryptForJournalistsOrNull(
        journalistsToMessagingKeys: Map<JournalistId, List<VerifiedSignedEncryptionKey>>,
        userKeyPair: EncryptionKeyPair,
        message: TwoPartyBox<EncryptableVector>,
        timestamp: Instant,
    ): DecryptedDeadDropMessage? {

        // for each journalist
        for (entry in journalistsToMessagingKeys.entries) {
            val journalistId = entry.key

            // and for all of their messaging keys
            for (remoteKey in entry.value) {

                try {
                    // if we succeed to decrypt a message for a key, there is no need to try any
                    // of the others
                    return tryDecryptAndParseMessage(
                        remoteKey = remoteKey,
                        userKeyPair = userKeyPair,
                        message = message,
                        remoteId = journalistId,
                        timestamp = timestamp,
                    )
                } catch (ignore: IllegalStateException) {
                    // a `IllegalStateException` indicates a general decryption error
                    // (wrong keys, ciphertext, ...); we expect that for most messages and
                    // hence do not take any action here
                }
            } // all messaging keys
        } // all journalists

        return null
    }

    @kotlin.jvm.Throws(java.lang.IllegalStateException::class)
    private fun tryDecryptAndParseMessage(
        remoteKey: VerifiedSignedEncryptionKey,
        userKeyPair: EncryptionKeyPair,
        message: TwoPartyBox<EncryptableVector>,
        remoteId: JournalistId,
        timestamp: Instant,
    ): DecryptedDeadDropMessage {
        val decryptedBytes = TwoPartyBox.decrypt(
            libSodium = libSodium,
            senderPk = remoteKey.pk,
            recipientSk = userKeyPair.secretEncryptionKey,
            data = message,
            constructor = ::EncryptableVector
        )

        return DecryptedDeadDropMessage.parse(
            bytes = decryptedBytes.asUnencryptedBytes(),
            remoteId = remoteId,
            timestamp = timestamp
        )
    }
}
