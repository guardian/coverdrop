package com.theguardian.coverdrop.core.integrationtests

import android.app.Application
import android.content.Context
import androidx.test.platform.app.InstrumentationRegistry
import com.theguardian.coverdrop.core.CoverDropLibInternalFixture
import com.theguardian.coverdrop.core.api.models.mostRecentMessagingKeyForEachCoverNode
import com.theguardian.coverdrop.core.api.models.mostRecentMessagingKeyForJournalist
import com.theguardian.coverdrop.core.createLibSodium
import com.theguardian.coverdrop.core.persistence.CoverDropFileManager
import com.theguardian.coverdrop.core.persistence.CoverDropNamespace
import com.theguardian.coverdrop.core.persistence.PublicStorage
import com.theguardian.coverdrop.testutils.InstantSubject
import com.theguardian.coverdrop.testutils.IntegrationTestVectors
import com.theguardian.coverdrop.testutils.TestApiCallProvider
import com.theguardian.coverdrop.testutils.TestClock
import com.theguardian.coverdrop.testutils.TestScenario
import com.theguardian.coverdrop.testutils.createCoverDropConfigurationForTest
import kotlinx.coroutines.runBlocking
import org.junit.Before
import org.junit.Test
import java.time.Duration
import java.time.Instant


class KeyRotationsScenario {

    private val context: Context = InstrumentationRegistry.getInstrumentation().targetContext
    private val scenario = TestScenario.KeyRotations

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

    @Before
    fun setup() {
        // Ensure that there are no timestamps that e.g. would prevent the background job to run
        publicStorage.deleteAll()
    }

    @Test
    fun testKeyRotationsScenario(): Unit = runBlocking {
        suspend fun runAppStart(filename: String) {
            testClock.setNow(nowOverride = testVectors.getNow(filename))
            testApiCallProvider.setPublicKeysFileName(filename)
            publicDataRepository.initialize()
        }

        // app starts for the first time
        runAppStart("001_initial.json")

        publicDataRepository.getVerifiedKeys().apply {
            InstantSubject
                .assertThat(
                    mostRecentMessagingKeyForJournalist(
                        "static_test_journalist",
                        testClock
                    ).notValidAfter
                )
                .isCloseTo(Instant.parse("2023-10-18T19:00:00Z"), tolerance = Duration.ofHours(1))
            InstantSubject
                .assertThat(mostRecentMessagingKeyForEachCoverNode(testClock)["covernode_001"]?.notValidAfter)
                .isCloseTo(Instant.parse("2023-10-18T19:00:00Z"), tolerance = Duration.ofHours(1))
        }

        // restarting after the first key rotation (covernode)
        runAppStart("002_covernode_msg_rotated_1.json")

        publicDataRepository.getVerifiedKeys().apply {
            InstantSubject
                .assertThat(mostRecentMessagingKeyForEachCoverNode(testClock)["covernode_001"]?.notValidAfter)
                .isCloseTo(Instant.parse("2023-10-26T19:00:00Z"), tolerance = Duration.ofHours(1))
        }

        // restarting after the second key rotation (journalist and covernode)
        runAppStart("003_covernode_msg_rotated_2.json")

        publicDataRepository.getVerifiedKeys().apply {
            InstantSubject
                .assertThat(mostRecentMessagingKeyForEachCoverNode(testClock)["covernode_001"]?.notValidAfter)
                .isCloseTo(Instant.parse("2023-11-01T19:00:00Z"), tolerance = Duration.ofHours(1))
        }
    }
}
