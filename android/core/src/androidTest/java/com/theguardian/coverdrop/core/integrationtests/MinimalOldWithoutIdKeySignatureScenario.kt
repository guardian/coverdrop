package com.theguardian.coverdrop.core.integrationtests

import android.app.Application
import android.content.Context
import androidx.test.platform.app.InstrumentationRegistry
import com.google.common.truth.Truth.assertThat
import com.theguardian.coverdrop.core.CoverDropLibInternalFixture
import com.theguardian.coverdrop.core.api.GsonApiJsonAdapter
import com.theguardian.coverdrop.core.api.models.mostRecentMessagingKeyForEachCoverNode
import com.theguardian.coverdrop.core.api.models.mostRecentMessagingKeyForJournalist
import com.theguardian.coverdrop.core.createLibSodium
import com.theguardian.coverdrop.core.persistence.CoverDropFileManager
import com.theguardian.coverdrop.core.persistence.CoverDropNamespace
import com.theguardian.coverdrop.core.persistence.PublicStorage
import com.theguardian.coverdrop.testutils.IntegrationTestVectors
import com.theguardian.coverdrop.testutils.TestApiCallProvider
import com.theguardian.coverdrop.testutils.TestClock
import com.theguardian.coverdrop.testutils.TestScenario
import com.theguardian.coverdrop.testutils.createCoverDropConfigurationForTest
import kotlinx.coroutines.runBlocking
import org.junit.Before
import org.junit.Test


/**
 * Backwards compatibility: published keys from before id keys carried a `signature` field must
 * still verify.
 */
class MinimalOldWithoutIdKeySignatureScenario {

    private val context: Context = InstrumentationRegistry.getInstrumentation().targetContext
    private val scenario = TestScenario.MinimalOldWithoutIdKeySignature

    private val testVectors = IntegrationTestVectors(context, scenario)
    private val testClock = TestClock(nowOverride = testVectors.getNow())

    private val fileManager = CoverDropFileManager(context, testClock, CoverDropNamespace.TEST)
    private val config = createCoverDropConfigurationForTest(
        context = context,
        scenario = scenario,
        clockOverride = testClock
    )

    private val publicStorage = PublicStorage(context, testClock, fileManager)
    private val testApiCallProvider = config.createApiCallProvider() as TestApiCallProvider
    private val encryptedStorage = createEncryptedStorageForTest(context, fileManager)

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
        publicStorage.deleteAll()
    }

    @Test
    fun testOldPublishedKeysWithoutSignature_thenVerify(): Unit = runBlocking {
        val publishedKeys = GsonApiJsonAdapter()
            .parsePublishedPublicKeys(testVectors.readJson("published_keys"))
            .keys
            .single()
        assertThat(publishedKeys.orgPk.signature).isNull()
        val journalistIdPk = publishedKeys.journalistsKeyHierarchy.single()
            .journalists.getValue("static_test_journalist").single().idPk
        assertThat(journalistIdPk.signature).isNull()

        publicDataRepository.initialize()

        val verifiedKeys = publicDataRepository.getVerifiedKeys()
        val hierarchy = verifiedKeys.keys.single()
        assertThat(hierarchy.journalistsHierarchies.single().journalists.getValue("static_test_journalist")).hasSize(1)
        assertThat(hierarchy.coverNodeHierarchies.single().coverNodes.getValue("covernode_001")).hasSize(1)

        val journalistMsgKey = verifiedKeys.mostRecentMessagingKeyForJournalist("static_test_journalist", testClock)
        assertThat(journalistMsgKey.notValidAfter).isGreaterThan(testClock.now())
        assertThat(verifiedKeys.mostRecentMessagingKeyForEachCoverNode(testClock)).containsKey("covernode_001")
    }
}
