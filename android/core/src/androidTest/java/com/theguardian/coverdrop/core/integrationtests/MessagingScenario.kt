package com.theguardian.coverdrop.core.integrationtests

import android.app.Application
import android.content.Context
import androidx.test.platform.app.InstrumentationRegistry
import com.google.common.truth.Truth.assertThat
import com.theguardian.coverdrop.core.CoverDropLibInternalFixture
import com.theguardian.coverdrop.core.LockState
import com.theguardian.coverdrop.core.background.CoverDropBackgroundJob
import com.theguardian.coverdrop.core.createLibSodium
import com.theguardian.coverdrop.core.crypto.EncryptionKeyPair
import com.theguardian.coverdrop.core.models.Message
import com.theguardian.coverdrop.core.persistence.CoverDropFileManager
import com.theguardian.coverdrop.core.persistence.CoverDropNamespace
import com.theguardian.coverdrop.core.persistence.PublicStorage
import com.theguardian.coverdrop.core.ui.models.DraftMessage
import com.theguardian.coverdrop.testutils.IntegrationTestVectors
import com.theguardian.coverdrop.testutils.TestApiCallProvider
import com.theguardian.coverdrop.testutils.TestClock
import com.theguardian.coverdrop.testutils.TestScenario
import com.theguardian.coverdrop.testutils.createCoverDropConfigurationForTest
import kotlinx.coroutines.runBlocking
import org.junit.Before
import org.junit.Test
import java.time.Duration


class MessagingScenario {

    private val context: Context = InstrumentationRegistry.getInstrumentation().targetContext
    private val scenario = TestScenario.Messaging


    private val testVectors = IntegrationTestVectors(context, scenario)
    private val testClock = TestClock(nowOverride = testVectors.getNow())

    // set-up working mocks for the underlying infrastructure components (ordered as they are
    // checked and used in the code under test)
    private val fileManager = CoverDropFileManager(context, CoverDropNamespace.TEST)
    private val config = createCoverDropConfigurationForTest(
        context = context,
        scenario = scenario,
        clockOverride = testClock
    )

    private val publicStorage = PublicStorage(context, fileManager)
    private val testApiCallProvider = config.createApiCallProvider() as TestApiCallProvider
    private val encryptedStorage = createEncryptedStorageForTest(context, fileManager)

    // initialize the real library components with the mocks
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
    fun testMessagingScenario(): Unit = runBlocking {
        suspend fun onAppStart() {
            publicDataRepository.initialize()
            encryptedStorage.onAppStart()
        }

        // app starts for the first time
        onAppStart()

        //
        // Setup and send initial user to journalist message
        //

        // user uses CoverDrop for the first time and creates a new encrypted storage using the
        // generated passphrase
        val passphrase = privateDataRepository.generatePassphrase()
        privateDataRepository.createOrResetStorage(passphrase)
        assertThat(privateDataRepository.getLockState()).isEqualTo(LockState.UNLOCKED)

        // we override the user key to the integration test user key
        val testUserKeyPair = testVectors.getKeys().getUserKeyPair()
        val userKeyPair = EncryptionKeyPair.newFromHexStrings(
            publicKey = testUserKeyPair.publicKey.key,
            secretKey = testUserKeyPair.secretKey
        )
        privateDataRepository.setUserKeyPair(userKeyPair)

        // user sends a message to the journalist
        val userMessage = "This is a test message from the user to the journalist"
        val journalistId = "static_test_journalist"
        privateDataRepository.createNewConversation(
            id = journalistId,
            message = DraftMessage(text = userMessage)
        )
        assertSendingQueueRealMessageCount(publicDataRepository, privateDataRepository, 1)

        // lock session and run background tasks that will send the message
        privateDataRepository.lock()
        CoverDropBackgroundJob.run(lib = lib)
        onAppStart()

        // return to session and check that message is sent
        privateDataRepository.unlock(passphrase)
        val conversationsAfterSending = privateDataRepository.getActiveConversation()!!
        assertThat(conversationsAfterSending.messages).hasSize(1)
        assertThat(conversationsAfterSending.messages[0]).isInstanceOf(Message.Sent::class.java)
        assertSendingQueueRealMessageCount(publicDataRepository, privateDataRepository, 0)

        privateDataRepository.lock()

        //
        // Download journalist reply
        //

        // advance the backend to the dead-drop that contains the journalist reply
        testApiCallProvider.setUserDeadDropsFileName("003_journalist_replied_and_processed.json")

        // travel into the future so that the download job runs again
        testClock.advance(config.minimumDurationBetweenDefaultDownloads + Duration.ofSeconds(1))
        onAppStart()

        // the unlock will parse the newly downloaded cached dead-drop
        privateDataRepository.unlock(passphrase)

        val conversationAfterDownload = privateDataRepository.getActiveConversation()!!
        assertThat(conversationAfterDownload.messages).hasSize(2)
        assertThat(conversationAfterDownload.messages[0]).isInstanceOf(Message.Sent::class.java)
        assertThat(conversationAfterDownload.messages[1]).isInstanceOf(Message.Received::class.java)
        assertSendingQueueRealMessageCount(publicDataRepository, privateDataRepository, 0)

        val receivedMessage = conversationAfterDownload.messages[1] as Message.Received
        assertThat(receivedMessage.message).isEqualTo("This is a test message from the journalist to the user")

        privateDataRepository.lock()

        //
        // Re-running the download and decrypting it does not yield duplicate messages
        //

        // travel into the future so that the download job runs again
        testClock.advance(config.minimumDurationBetweenDefaultDownloads + Duration.ofSeconds(1))
        onAppStart()

        // the unlock will parse the newly downloaded cached dead-drop
        privateDataRepository.unlock(passphrase)

        val conversationAfterReDownload = privateDataRepository.getActiveConversation()!!
        assertThat(conversationAfterReDownload.messages).hasSize(2)
    }

    @Test
    fun testMessageExpiry() = runBlocking {
        suspend fun onAppStart() {
            publicDataRepository.initialize()
            encryptedStorage.onAppStart()
        }

        // app starts for the first time
        onAppStart()

        //
        // Setup and send initial user to journalist message
        //

        // user uses CoverDrop for the first time and creates a new encrypted storage using the
        // generated passphrase
        val passphrase = privateDataRepository.generatePassphrase()
        privateDataRepository.createOrResetStorage(passphrase)
        assertThat(privateDataRepository.getLockState()).isEqualTo(LockState.UNLOCKED)

        // we override the user key to the integration test user key
        val testUserKeyPair = testVectors.getKeys().getUserKeyPair()
        val userKeyPair = EncryptionKeyPair.newFromHexStrings(
            publicKey = testUserKeyPair.publicKey.key,
            secretKey = testUserKeyPair.secretKey
        )
        privateDataRepository.setUserKeyPair(userKeyPair)

        // user sends a message to the journalist
        val userMessage = "This is a test message from the user to the journalist"
        val journalistId = "static_test_journalist"
        privateDataRepository.createNewConversation(
            id = journalistId,
            message = DraftMessage(text = userMessage)
        )
        assertSendingQueueRealMessageCount(publicDataRepository, privateDataRepository, 1)

        // lock session and run background tasks that will send the message
        privateDataRepository.lock()
        CoverDropBackgroundJob.run(lib = lib)
        onAppStart()

        // return to session and check that message is sent
        privateDataRepository.unlock(passphrase)
        val conversationsAfterSending = privateDataRepository.getActiveConversation()!!
        assertThat(conversationsAfterSending.messages).hasSize(1)
        assertThat(conversationsAfterSending.messages[0]).isInstanceOf(Message.Sent::class.java)
        assertSendingQueueRealMessageCount(publicDataRepository, privateDataRepository, 0)

        privateDataRepository.lock()

        //
        // Verify that message is no longer persisted after the expiry time
        //

        // travel into the future so that the message expires
        testClock.advance(config.messageExpiryDuration + Duration.ofSeconds(1))

        privateDataRepository.unlock(passphrase)
        val conversationsAfterExpiryRemoval = privateDataRepository.getActiveConversation()!!
        assertThat(conversationsAfterExpiryRemoval.messages).isEmpty()
        privateDataRepository.lock()
    }
}
