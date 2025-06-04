package com.theguardian.coverdrop.core

import androidx.annotation.VisibleForTesting
import com.theguardian.coverdrop.core.api.models.VerifiedKeys
import com.theguardian.coverdrop.core.api.models.mostRecentMessagingKeyForEachCoverNode
import com.theguardian.coverdrop.core.api.models.mostRecentMessagingKeyForJournalist
import com.theguardian.coverdrop.core.crypto.DeadDropProcessor
import com.theguardian.coverdrop.core.crypto.EncryptionKeyPair
import com.theguardian.coverdrop.core.crypto.Passphrase
import com.theguardian.coverdrop.core.crypto.PrivateSendingQueueHint
import com.theguardian.coverdrop.core.encryptedstorage.IEncryptedStorageSession
import com.theguardian.coverdrop.core.models.*
import com.theguardian.coverdrop.core.persistence.MailboxContent
import com.theguardian.coverdrop.core.persistence.StoredMessage
import com.theguardian.coverdrop.core.persistence.StoredMessageThread
import com.theguardian.coverdrop.core.persistence.copyWithNewMessage
import com.theguardian.coverdrop.core.ui.models.DraftMessage
import com.theguardian.coverdrop.core.utils.IClock

enum class LockState { LOCKED, UNLOCKED }

/**
 * A fallback Journalist that is used when the real Journalist cannot be found (e.g. they have
 * been removed from the published list). This is used to avoid crashes and provide at least a safe
 * read-only view.
 */
internal val MissingJournalistInfo = JournalistInfo(
    id = "MissingNo.",
    displayName = "Journalist",
    sortName = "Journalist",
    description = "",
    isTeam = false,
    tag = "",
    visibility = JournalistVisibility.HIDDEN,
)

interface ICoverDropPrivateDataRepository {
    /**
     * Generates and returns a new passphrase taking into account the device's SE availability. The
     * passphrase can then used with [createOrResetStorage].
     */
    suspend fun generatePassphrase(): Passphrase

    /**
     * Returns the number of words in all passphrases that are generated on this device. The
     * returned number is guaranteed to remain constant across updates and app restarts.
     */
    fun getPassphraseWordCount(): Int

    /**
     * This replaces the encrypted storage (if any) with one that is encrypted with the given
     * passphrase. This is typically done within the "New Session" flow and should only done after
     * explicit user confirmation.
     *
     * @throws EncryptedStorageBadPassphraseException if the passphrase cannot have been possibly
     * created using [generatePassphrase]. This typically indicates spelling errors.
     */
    suspend fun createOrResetStorage(passphrase: Passphrase)

    /**
     * Unlocks the existing encrypted storage with the given passphrase.
     *
     * @throws EncryptedStorageBadPassphraseException if the passphrase cannot have been possibly
     * created using [generatePassphrase]. This typically indicates spelling errors.
     *
     * @throws EncryptedStorageAuthenticationFailed if the passphrase fails to unlock the storage.
     * This means that the encrypted storage has not been initialized before of a different
     * passphrase has been used.
     */
    suspend fun unlock(passphrase: Passphrase)

    /**
     * Locks the existing encrypted storage.
     */
    suspend fun lock()

    /**
     * Returns the current lock state. Use the [CoverDropLib.getLockState] method for a [StateFlow]
     * of this property that can be easily used in UI implementations.
     */
    fun getLockState(): LockState

    /**
     * Creates a new conversation with the journalist (or team) with the given [id].
     *
     * If there is already a conversation for the given [id], the message is appended as it would
     * be when calling [replyToConversation]. Otherwise, a new conversation is created which will
     * become the currently active conversation.
     */
    suspend fun createNewConversation(id: JournalistId, message: DraftMessage)

    /**
     * Returns the currently active conversation (if any).
     */
    suspend fun getActiveConversation(): MessageThread?

    /**
     * Returns the (potentially empty) list of currently inactive conversations.
     */
    suspend fun getInactiveConversations(): List<MessageThread>

    /**
     * Returns the conversation for the given [id] or throws if none exists.
     */
    suspend fun getConversationForId(id: JournalistId): MessageThread?

    /**
     * Replies with the given [message] to the assumed existing conversation with [id]. The
     * [message] should not have a subject set.
     */
    suspend fun replyToConversation(id: JournalistId, message: DraftMessage)

    /**
     * Deletes the message vault. This will set a new random passphrase, overwrite the existing
     * storage, and then move to a locked state.
     */
    suspend fun deleteVault()
}

class CoverDropPrivateDataRepository internal constructor(lib: ICoverDropLibInternal) :
    ICoverDropPrivateDataRepository {

    private var lockState = LockState.LOCKED
    private val publishLockState = lib::publishLockState

    private var mailboxContent: MailboxContent? = null

    private val encryptedStorage = lib.getEncryptedStorage()
    private var encryptedStorageActiveSession: IEncryptedStorageSession? = null

    private val clock = lib.getClock()
    private val config = lib.getConfig()
    private val libSodium = lib.getLibSodium()
    private val protocol = lib.getProtocol()
    private val publicDataRepository = lib.getPublicDataRepository()

    override suspend fun generatePassphrase() = encryptedStorage.generateNewRandomPassphrase()

    override suspend fun createOrResetStorage(passphrase: Passphrase) {
        lock()

        // store an empty mailbox content using a new active session which will override any
        // previous data
        encryptedStorageActiveSession = encryptedStorage.createOrResetStorage(passphrase)

        // once created, we can "unlock" which is essentially the reverse of the above
        unlock(passphrase)
    }

    override suspend fun unlock(passphrase: Passphrase) {
        require(lockState == LockState.LOCKED)

        encryptedStorageActiveSession = encryptedStorage.unlockSession(passphrase)
        val decryptedMailbox = encryptedStorage.loadFromStorage(encryptedStorageActiveSession!!)

        // merge the dead drops into the mailbox content
        val deadDropProcessor = DeadDropProcessor(libSodium)
        val downloadedDeadDrops = publicDataRepository.getMostRecentDeadDrops()
        val newStoredMessageThreads = deadDropProcessor.decryptAndMerge(
            existingMessageThreads = decryptedMailbox.getMessageThreads(),
            deadDrops = downloadedDeadDrops,
            journalistsKeyHierarchies = publicDataRepository.getVerifiedKeys().keys.flatMap { it.journalistsHierarchies },
            userKeyPair = decryptedMailbox.encryptionKeyPair,
        )

        // update state and persist
        mailboxContent = decryptedMailbox.copy(messageThreads = newStoredMessageThreads)
        save(mailboxContent!!)

        lockState = LockState.UNLOCKED
        publishLockState(LockState.UNLOCKED)
    }

    override suspend fun lock() {
        mailboxContent = null
        lockState = LockState.LOCKED
        publishLockState(LockState.LOCKED)
    }

    override fun getLockState() = lockState

    override fun getPassphraseWordCount(): Int = encryptedStorage.getPassphraseWordCount()

    override suspend fun createNewConversation(
        id: JournalistId,
        message: DraftMessage,
    ) {
        // creating a new conversation is the same as replying to a non-existing conversation
        replyToConversation(id, message)
    }

    override suspend fun getActiveConversation(): MessageThread? {
        require(lockState == LockState.UNLOCKED)
        val mailbox = mailboxContent!!

        // return the one with the most recent update
        val storedMessageThread = mailbox.getMessageThreads()
            .maxByOrNull { it.mostRecentUpdate() }

        return storedMessageThread?.let { mapStoredThreadToThread(it) }
    }

    override suspend fun getInactiveConversations(): List<MessageThread> {
        require(lockState == LockState.UNLOCKED)
        val mailbox = mailboxContent!!

        // return all except the most recent thread
        var storedMessageThreads = mailbox.getMessageThreads()
            .sortedByDescending { it.mostRecentUpdate() }
        if (storedMessageThreads.isNotEmpty()) {
            storedMessageThreads = storedMessageThreads.drop(1)
        }

        return storedMessageThreads.map { mapStoredThreadToThread(it) }
    }

    override suspend fun getConversationForId(id: JournalistId): MessageThread? {
        require(lockState == LockState.UNLOCKED)
        val mailbox = mailboxContent!!

        val thread = mailbox.messageThreads.firstOrNull { it.recipientId == id } ?: return null
        return mapStoredThreadToThread(thread)
    }

    override suspend fun replyToConversation(id: JournalistId, message: DraftMessage) {
        require(lockState == LockState.UNLOCKED)
        val mailbox = mailboxContent!!

        // validate message; this might throw
        message.validateOrThrow()

        // send message which will add it to the private sending queue
        val privateSendingQueueHint = encryptAndSendMessage(id, mailbox, message, clock)

        val newMessage = StoredMessage.local(
            timestamp = clock.now(),
            message = message.text,
            privateSendingQueueHint = privateSendingQueueHint,
        )

        // create a new thread if none exists or append to the existing one
        val mayBeExistingThread = mailbox.getThreadWithId(id)
        val newThread = mayBeExistingThread?.copyWithNewMessage(newMessage) ?: StoredMessageThread(
            recipientId = id,
            messages = listOf(newMessage)
        )

        // copy the mailbox with the new thread
        val newMailboxContent = mailbox.copyWithNewThread(newThread)
        save(newMailboxContent)
    }

    override suspend fun deleteVault() {
        require(lockState == LockState.UNLOCKED)
        val mailbox = mailboxContent!!

        publicDataRepository.clearQueue(mailbox.privateSendingQueueSecret)

        val randomPassphrase = encryptedStorage.generateNewRandomPassphrase()
        encryptedStorage.createOrResetStorage(randomPassphrase)

        lock()
    }

    /**
     * Used in the integration tests to set the user key pair that allows to decrypt ingoing
     * messages.
     */
    @VisibleForTesting
    internal fun setUserKeyPair(userKeyPair: EncryptionKeyPair) {
        val newMailboxContent = mailboxContent!!.copy(encryptionKeyPair = userKeyPair)
        save(newMailboxContent)
    }

    /**
     * Used in the integration tests to assert the current state of the messages and non-exposed
     * details such as their PSQ hints.
     */
    @VisibleForTesting
    internal fun getMailboxContent() = mailboxContent

    /**
     * Maps a [StoredMessageThread] to a [MessageThread]. Importantly, the [MessageThread] is
     * populated with the [JournalistInfo] based on the [StoredMessageThread.recipientId]. If the
     * [JournalistInfo] cannot be found, the [MissingJournalistInfo] is used instead.
     */
    private suspend fun mapStoredThreadToThread(storedMessageThread: StoredMessageThread): MessageThread {
        val journalistInfo = publicDataRepository.getAllJournalists(includeHidden = true)
            .find { info -> info.id == storedMessageThread.recipientId } ?: MissingJournalistInfo

        val privateSendingQueueHints =
            HashSet(publicDataRepository.getPrivateSendingQueueHintsInQueue())

        val messages = storedMessageThread.messages.map { thread ->
            mapStoredMessageToMessage(thread, privateSendingQueueHints)
        }

        return MessageThread(journalistInfo, messages)
    }

    /**
     * Maps a [StoredMessage] to a [Message]. Importantly, the [MessageType] is determined based on
     * whether the message is remote or not and whether its [PrivateSendingQueueHint] is in the
     * [privateSendingQueueHints].
     */
    private fun mapStoredMessageToMessage(
        storedMessage: StoredMessage,
        privateSendingQueueHints: HashSet<PrivateSendingQueueHint>,
    ): Message {
        val isPending = privateSendingQueueHints.contains(storedMessage.privateSendingQueueHint)
        return Message.fromStored(storedMessage = storedMessage, isPending = isPending)
    }

    /**
     * Saves the given [MailboxContent] to the encrypted storage. This method also removes messages
     * that are older than the message expiry duration. This method is called after every change to
     * the mailbox content.
     */
    private fun save(newMailboxContent: MailboxContent) {
        // when saving, remove messages older than the message expiry duration
        val cutoff = clock.now() - config.messageExpiryDuration
        val truncatedMailboxContent = newMailboxContent.copyMinusOldMessages(cutoff = cutoff)

        // then save the messages (note that this might truncate more messages, e.g. if they exceed
        // the maximum mailbox size)
        encryptedStorage.saveToStorage(encryptedStorageActiveSession!!, truncatedMailboxContent)
        mailboxContent = truncatedMailboxContent
    }

    /**
     * Encrypts the [DraftMessage] using the [VerifiedKeys] from the
     * [publicDataRepository] and then schedules the resulting message to be sent via the
     * [PrivateSendingQueue].
     */
    private suspend fun encryptAndSendMessage(
        id: JournalistId,
        mailbox: MailboxContent,
        message: DraftMessage,
        clock: IClock,
    ): PrivateSendingQueueHint {
        val verifiedKeys = publicDataRepository.getVerifiedKeys()

        // Find the tag and the most recent key for the given journalist
        val journalistMsgKey = verifiedKeys.mostRecentMessagingKeyForJournalist(id, clock)
        val journalistTag = requireNotNull(publicDataRepository.getJournalistTag(id)) {
            "Journalist tag not found for $id"
        }

        // Find the most recent key for each coverNode
        val coverNodesToMostRecentKey =
            verifiedKeys.mostRecentMessagingKeyForEachCoverNode(clock)
        require(coverNodesToMostRecentKey.isNotEmpty()) { "No valid covernode key candidate found at all" }

        val encryptedMessage = protocol.encryptUserToJournalistMessageViaCoverNode(
            coverNodesToMostRecentKey = coverNodesToMostRecentKey,
            journalistMsgKey = journalistMsgKey,
            userMsgKey = mailbox.encryptionKeyPair.publicEncryptionKey,
            paddedMessage = PaddedCompressedString.fromString(message.text),
            journalistTag = journalistTag
        )

        return publicDataRepository.addMessageToQueue(
            privateSendingQueueSecret = mailbox.privateSendingQueueSecret,
            message = encryptedMessage
        )
    }

}

