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
 * Published identity keys from before the required `signature` field are no longer accepted.
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
    fun testOldPublishedKeysWithoutIdKeySignature_thenDropsIdentityKeys(): Unit = runBlocking {
        val publishedKeys = GsonApiJsonAdapter()
            .parsePublishedPublicKeys(testVectors.readJson("published_keys"))
            .keys
            .single()
        assertThat(publishedKeys.orgPk.signature).isNull()
        val journalistIdPk = publishedKeys.journalistsKeyHierarchy.single()
            .journalists.getValue("static_test_journalist").single().idPk
        assertThat(journalistIdPk.signature).isNull()
        val coverNodeIdPk = publishedKeys.coverNodesKeyHierarchy.single()
            .coverNodes.getValue("covernode_001").single().idPk
        assertThat(coverNodeIdPk.signature).isNull()

        var initializationFailed = false
        try {
            publicDataRepository.initialize()
        } catch (e: IllegalArgumentException) {
            initializationFailed = true
            assertThat(e).hasMessageThat().contains("No valid covernode key candidate found at all")
        }
        assertThat(initializationFailed).isTrue()

        val verifiedKeys = publicDataRepository.getVerifiedKeys()
        val hierarchy = verifiedKeys.keys.single()
        assertThat(hierarchy.journalistsHierarchies.single().journalists.getValue("static_test_journalist")).isEmpty()
        assertThat(hierarchy.coverNodeHierarchies.single().coverNodes.getValue("covernode_001")).isEmpty()
    }
}
