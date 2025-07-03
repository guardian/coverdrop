package com.theguardian.coverdrop.core

import android.app.Application
import android.content.Context
import androidx.test.platform.app.InstrumentationRegistry
import com.google.common.truth.Truth.assertThat
import com.theguardian.coverdrop.core.background.CoverDropBackgroundJob
import com.theguardian.coverdrop.core.integrationtests.assertSendingQueueRealMessageCount
import com.theguardian.coverdrop.core.integrationtests.createEncryptedStorageForTest
import com.theguardian.coverdrop.core.models.Message
import com.theguardian.coverdrop.core.persistence.CoverDropFileManager
import com.theguardian.coverdrop.core.persistence.CoverDropNamespace
import com.theguardian.coverdrop.core.persistence.PublicStorage
import com.theguardian.coverdrop.core.ui.models.DraftMessage
import com.theguardian.coverdrop.testutils.TestApiCallProvider
import com.theguardian.coverdrop.testutils.TestScenario
import com.theguardian.coverdrop.testutils.createCoverDropConfigurationForTest
import kotlinx.coroutines.runBlocking
import org.junit.Before
import org.junit.Test


class CoverDropRepositoriesTest {

    val context: Context = InstrumentationRegistry.getInstrumentation().targetContext

    // set-up working mocks for the underlying infrastructure components (ordered as they are
    // checked and used in the code under test)
    private val config = createCoverDropConfigurationForTest(context, TestScenario.Minimal)
    private val fileManager = CoverDropFileManager(context, config.clock, CoverDropNamespace.TEST)
    private val publicStorage = PublicStorage(context, config.clock, fileManager)
    private val testApiCallProvider = config.createApiCallProvider() as TestApiCallProvider
    private val encryptedStorage = createEncryptedStorageForTest(context, fileManager)

    // initialize the main library components with the mocks
    private val lib = CoverDropLibInternalFixture(
        mApiCallProvider = testApiCallProvider,
        mContext = context.applicationContext as Application,
        mConfig = config,
        mClock = config.clock,
        mEncryptedStorage = encryptedStorage,
        mLibSodium = createLibSodium(),
        mPublicStorage = publicStorage,
    )
    private val publicDataRepository = lib.getPublicDataRepository()
    private val privateDataRepository = lib.getPrivateDataRepository()

    @Before
    fun setup() {
        // Ensure that there are no timestamps that e.g. would prevent the background job to run
        publicStorage.deleteAll()
    }

    @Test
    fun testHappyPath(): Unit = runBlocking {
        suspend fun onAppStart() {
            publicDataRepository.initialize()
            encryptedStorage.onAppStart()
        }

        // app starts for the first time
        onAppStart()

        // user uses CoverDrop for the first time and creates a new encrypted storage using the
        // generated passphrase
        val passphrase = privateDataRepository.generatePassphrase()
        privateDataRepository.createOrResetStorage(passphrase)
        assertThat(privateDataRepository.getLockState()).isEqualTo(LockState.UNLOCKED)

        // user composes a new message and sends it to the first journalist they can find
        val journalistId = publicDataRepository.getAllJournalists().first().id
        val firstMessage = DraftMessage(text = "Lorem Ipsum Yolo")
        privateDataRepository.createNewConversation(journalistId, firstMessage)
        assertSendingQueueRealMessageCount(publicDataRepository, privateDataRepository, 1)

        // the message should now be in `PENDING` state
        val conversationsBeforeSending = privateDataRepository.getActiveConversation()!!
        assertThat(conversationsBeforeSending.recipient.id).isEqualTo(journalistId)
        val messageBeforeSending = conversationsBeforeSending.messages.first() as Message.Pending
        assertThat(messageBeforeSending).isInstanceOf(Message.Pending::class.java)
        assertThat(messageBeforeSending.message).contains(firstMessage.text)
        assertSendingQueueRealMessageCount(publicDataRepository, privateDataRepository, 1)

        // user locks the storage and navigates away from the app
        privateDataRepository.lock()
        assertThat(privateDataRepository.getLockState()).isEqualTo(LockState.LOCKED)

        // app goes to background and then is restarted
        testApiCallProvider.clearLoggedPostRequests()
        CoverDropBackgroundJob.run(lib = lib)
        onAppStart()

        assertThat(
            testApiCallProvider.getLoggedPostRequests()
                .filter { it.first.contains("/user/message") }
        ).isNotEmpty()

        // user logs back into CoverDrop
        privateDataRepository.unlock(passphrase)
        assertThat(privateDataRepository.getLockState()).isEqualTo(LockState.UNLOCKED)
        assertSendingQueueRealMessageCount(publicDataRepository, privateDataRepository, 0)

        // user reads the previous thread
        val conversationsAfterSending = privateDataRepository.getActiveConversation()!!
        assertThat(conversationsAfterSending.recipient.id).isEqualTo(journalistId)
        val messageAfterSending = conversationsAfterSending.messages.first() as Message.Sent
        assertThat(messageAfterSending).isInstanceOf(Message.Sent::class.java)
        assertThat(messageAfterSending.message).contains(firstMessage.text)

        // user replies to the thread with a second message
        val secondMessage = DraftMessage(text = "Oh actually...")
        privateDataRepository.replyToConversation(journalistId, secondMessage)
        assertSendingQueueRealMessageCount(publicDataRepository, privateDataRepository, 1)
    }
}
