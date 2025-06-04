package com.theguardian.coverdrop.ui.tests.utils

import com.theguardian.coverdrop.core.ICoverDropLib
import com.theguardian.coverdrop.core.ICoverDropPrivateDataRepository
import com.theguardian.coverdrop.core.ICoverDropPublicDataRepository
import com.theguardian.coverdrop.core.LockState
import com.theguardian.coverdrop.core.api.models.SystemStatus
import com.theguardian.coverdrop.core.crypto.Passphrase
import com.theguardian.coverdrop.core.crypto.PassphraseWordList
import com.theguardian.coverdrop.core.encryptedstorage.EncryptedStorageAuthenticationFailed
import com.theguardian.coverdrop.core.encryptedstorage.EncryptedStorageBadPassphraseException
import com.theguardian.coverdrop.core.models.DebugContext
import com.theguardian.coverdrop.core.models.JournalistId
import com.theguardian.coverdrop.core.models.JournalistInfo
import com.theguardian.coverdrop.core.models.JournalistVisibility
import com.theguardian.coverdrop.core.models.Message
import com.theguardian.coverdrop.core.models.MessageThread
import com.theguardian.coverdrop.core.models.StatusEvent
import com.theguardian.coverdrop.core.ui.models.DraftMessage
import com.theguardian.coverdrop.core.ui.models.UiPassphraseWord
import com.theguardian.coverdrop.core.ui.models.toUiPassphrase
import com.theguardian.coverdrop.ui.utils.COVERDROP_SAMPLE_DATA
import com.theguardian.coverdrop.ui.utils.SampleDataProvider
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import java.time.Instant
import java.util.EnumSet

internal val TEST_PASSPHRASE = COVERDROP_SAMPLE_DATA.getShortPassphrase()
internal val TEST_PASSPHRASE_LENGTH = TEST_PASSPHRASE.getWords().size

/**
 * Since we do not read the actual word list in tests, we use this special constant to trigger the
 * error case where the user enters an implausible passphrase word.
 */
internal val TEST_INVALID_WORD: String = "invalidword"

enum class MockedPassphraseBehavior {

    /**
     * Always return the same passphrase [COVERDROP_SAMPLE_DATA.getShortPassphrase].
     */
    SAMPLE,

    /**
     * Generate a random passphrase.
     */
    RANDOM;

    fun generatePassphrase(passphraseWordList: PassphraseWordList): Passphrase {
        return when (this) {
            SAMPLE -> COVERDROP_SAMPLE_DATA.getShortPassphrase()
            RANDOM -> passphraseWordList.generatePassphrase(TEST_PASSPHRASE_LENGTH)
        }
    }
}


/**
 * Mock for the [ICoverDropPrivateDataRepository] as injected by [CoverDropLibMock].
 *
 * The implementations of the methods is incomplete and will advance as the UI to test them is
 * progressing. Where possible we fail silently by not performing any action.
 */
class CoverDropPrivateDataRepositoryMock(lib: CoverDropLibMock) : ICoverDropPrivateDataRepository {

    /** Behaviours that can be simulated by the [CoverDropPrivateDataRepositoryMock]. */
    enum class SimulatedBehaviour {
        FAIL_ON_CREATE_NEW_CONVERSATION
    }

    private val publishLockState = lib::publishLockState
    private val passphraseWordList = lib.getPassphraseWordList()

    private var lockState = LockState.LOCKED
    private var mockedThreads = HashMap<JournalistId, MessageThread>()

    private var passphraseBehavior = MockedPassphraseBehavior.SAMPLE
    private var storedPassphrase = passphraseBehavior.generatePassphrase(passphraseWordList)

    private var activeSimulatedBehaviours = EnumSet.noneOf(SimulatedBehaviour::class.java)

    fun simulateBehaviour(behaviour: SimulatedBehaviour) {
        activeSimulatedBehaviours.add(behaviour)
    }

    fun clearSimulatedBehaviours() {
        activeSimulatedBehaviours.clear()
    }

    override suspend fun generatePassphrase(): Passphrase {
        return passphraseBehavior.generatePassphrase(passphraseWordList)
    }

    override suspend fun createOrResetStorage(passphrase: Passphrase) {
        storedPassphrase = passphrase
        unlock(passphrase)
    }

    override suspend fun unlock(passphrase: Passphrase) {
        require(lockState == LockState.LOCKED)

        if (passphrase.toUiPassphrase().contains(UiPassphraseWord(TEST_INVALID_WORD))) {
            throw EncryptedStorageBadPassphraseException()
        }

        if (passphrase != storedPassphrase) {
            throw EncryptedStorageAuthenticationFailed()
        }

        lockState = LockState.UNLOCKED
        publishLockState(LockState.UNLOCKED)
    }

    override suspend fun lock() {
        lockState = LockState.LOCKED
        publishLockState(LockState.LOCKED)
    }

    override fun getLockState() = lockState

    override suspend fun createNewConversation(id: JournalistId, message: DraftMessage) {
        if (activeSimulatedBehaviours.contains(SimulatedBehaviour.FAIL_ON_CREATE_NEW_CONVERSATION)) {
            throw RuntimeException("Failed to create a new conversation")
        }

        val recipientJournalist = COVERDROP_SAMPLE_DATA.getJournalists().firstOrNull { it.id == id }
        val recipientTeam = COVERDROP_SAMPLE_DATA.getTeams().firstOrNull { it.id == id }

        val thread = MessageThread(
            recipient = recipientJournalist ?: recipientTeam!!,
            messages = listOf(Message.pending(message.text)),
        )
        mockedThreads[id] = thread
    }

    override suspend fun getActiveConversation(): MessageThread? {
        return mockedThreads[getJournalistIdForLatestConversation()]
    }

    override suspend fun getInactiveConversations(): List<MessageThread> {
        return mockedThreads.values.toList()
            .filter { it.recipient.id != getJournalistIdForLatestConversation() }
    }

    private fun getJournalistIdForLatestConversation(): JournalistId? {
        return mockedThreads.values
            .sortedByDescending { it.messages.lastOrNull()?.timestamp }
            .firstOrNull()?.recipient?.id
    }

    override suspend fun getConversationForId(
        id: JournalistId,
    ): MessageThread {
        return mockedThreads[id]!!
    }

    override suspend fun replyToConversation(id: JournalistId, message: DraftMessage) {
        val thread = mockedThreads[id]!!
        val messages = thread.messages.toMutableList()
        messages.add(Message.pending(message.text))
        mockedThreads[id] = thread.copy(messages = messages)
    }

    override suspend fun deleteVault() {
        mockedThreads.clear()
        storedPassphrase = generatePassphrase()
        lock()
    }

    override fun getPassphraseWordCount() = TEST_PASSPHRASE_LENGTH

    fun addConversationForId(
        id: JournalistId,
        thread: MessageThread,
    ) {
        mockedThreads[id] = thread
    }

    fun setPassphraseBehavior(passphraseBehavior: MockedPassphraseBehavior) {
        this.passphraseBehavior = passphraseBehavior
    }
}

class CoverDropPublicDataRepositoryMock : ICoverDropPublicDataRepository {

    /** Behaviours that can be simulated by the [CoverDropPublicDataRepositoryMock]. */
    enum class SimulatedBehaviour {
        ONLY_ONE_JOURNALIST_AVAILABLE,
        NO_DEFAULT_JOURNALIST,
    }

    private var statusEvent = StatusEvent(
        status = SystemStatus.AVAILABLE,
        isAvailable = true,
        description = "Mocked available status",
    )

    private var activeSimulatedBehaviours = EnumSet.noneOf(SimulatedBehaviour::class.java)

    fun simulateBehaviour(behaviour: SimulatedBehaviour) {
        activeSimulatedBehaviours.add(behaviour)
    }

    fun clearSimulatedBehaviours() {
        activeSimulatedBehaviours.clear()
    }

    override suspend fun getAllJournalists(includeHidden: Boolean): List<JournalistInfo> {
        val all =
            if (activeSimulatedBehaviours.contains(SimulatedBehaviour.ONLY_ONE_JOURNALIST_AVAILABLE)) {
                COVERDROP_SAMPLE_DATA.getTeams().take(1)
            } else {
                COVERDROP_SAMPLE_DATA.getTeams() + COVERDROP_SAMPLE_DATA.getJournalists()
            }

        return all.filter { includeHidden || it.visibility == JournalistVisibility.VISIBLE }
    }

    override suspend fun getDefaultJournalist(): JournalistInfo? {
        return if (activeSimulatedBehaviours.contains(SimulatedBehaviour.NO_DEFAULT_JOURNALIST)) {
            null
        } else {
            getAllJournalists().first()
        }
    }

    override suspend fun getStatusEvent() = statusEvent

    fun setStatusEvent(statusEvent: StatusEvent) {
        this.statusEvent = statusEvent
    }
}

class CoverDropLibMock : ICoverDropLib {
    private val publicDataRepositoryMock = CoverDropPublicDataRepositoryMock()
    private val privateDataRepositoryMock = CoverDropPrivateDataRepositoryMock(this)
    private val lockStateFlow = MutableSharedFlow<LockState>(replay = 3)

    override fun getPrivateDataRepository(): CoverDropPrivateDataRepositoryMock {
        return privateDataRepositoryMock
    }

    override fun getPublicDataRepository(): ICoverDropPublicDataRepository {
        return publicDataRepositoryMock
    }

    override fun getInitializationSuccessful(): MutableStateFlow<Boolean> = MutableStateFlow(true)

    override fun getInitializationFailed(): StateFlow<Boolean> = MutableStateFlow(false)

    override fun getPassphraseWordList() = PassphraseWordList(
        wordList = SampleDataProvider().getWordList()
    )

    override fun getLockFlow() = lockStateFlow

    fun publishLockState(lockState: LockState) {
        lockStateFlow.tryEmit(lockState)
    }

    override suspend fun forceRefreshInLocalTestMode() {
        TODO("Not yet implemented")
    }

    override fun getDebugContext(): DebugContext {
        return DebugContext(
            lastUpdatePublicKeys = Instant.now(),
            lastUpdateDeadDrops = Instant.now(),
            lastBackgroundTry = Instant.now(),
            lastBackgroundSend = Instant.now(),
            hashedOrgKey = "[abcdef ghijkl mnoqpr stu]"
        )
    }
}
