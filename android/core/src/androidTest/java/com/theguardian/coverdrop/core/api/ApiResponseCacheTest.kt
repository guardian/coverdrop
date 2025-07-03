package com.theguardian.coverdrop.core.api

import androidx.test.platform.app.InstrumentationRegistry
import com.google.common.truth.Truth.assertThat
import com.theguardian.coverdrop.core.api.models.PublishedJournalistToUserDeadDrop
import com.theguardian.coverdrop.core.api.models.PublishedJournalistToUserDeadDropsList
import com.theguardian.coverdrop.core.api.models.PublishedKeysAndProfiles
import com.theguardian.coverdrop.core.api.models.PublishedSignedSigningKey
import com.theguardian.coverdrop.core.crypto.ED25519_PUBLIC_KEY_BYTES
import com.theguardian.coverdrop.core.mocks.CoverDropApiClientMock
import com.theguardian.coverdrop.core.persistence.CoverDropFileManager
import com.theguardian.coverdrop.core.persistence.CoverDropNamespace
import com.theguardian.coverdrop.core.persistence.PublicStorage
import com.theguardian.coverdrop.core.utils.hexEncode
import com.theguardian.coverdrop.testutils.InstantSubject
import com.theguardian.coverdrop.testutils.IntegrationTestVectors
import com.theguardian.coverdrop.testutils.TestApiCallProvider
import com.theguardian.coverdrop.testutils.TestClock
import com.theguardian.coverdrop.testutils.TestScenario
import com.theguardian.coverdrop.testutils.createMinimalCoverDropTestConfigurationWithClock
import kotlinx.coroutines.runBlocking
import org.junit.Before
import org.junit.Test
import java.time.Duration
import java.time.Instant


class ApiResponseCacheTest {

    private val context = InstrumentationRegistry.getInstrumentation().context
    private val clock = TestClock(nowOverride = Instant.now())
    private val configuration = createMinimalCoverDropTestConfigurationWithClock(clock)
    private val fileManager = CoverDropFileManager(context, clock, CoverDropNamespace.TEST)
    private val publicStorage = PublicStorage(
        context = context,
        clock = clock,
        fileManager = fileManager
    )

    private val apiCallProvider =
        TestApiCallProvider(IntegrationTestVectors(context, TestScenario.Minimal))
    private val apiClient = CoverDropApiClientMock.fromApiCallProvider(
        apiCallProvider = apiCallProvider,
        apiConfiguration = configuration.apiConfiguration,
    )
    private val instance = ApiResponseCache(apiClient, publicStorage, configuration, clock)

    @Before
    fun setUp() {
        publicStorage.deleteAll()
    }

    @Test
    fun testPublishedKeys_whenDownloadedWithEmptyStorage_thenAvailableAsMostRecent() {
        assertThat(publicStorage.readPublishedKeys()).isNull()

        runBlocking {
            instance.downloadAndUpdateCachedPublishedKeys()
        }

        assertThat(publicStorage.readPublishedKeys()).isNotNull()
        InstantSubject.assertThat(publicStorage.readPublishedKeysLastUpdate())
            .isCloseTo(clock.now())
    }

    @Test
    fun testPublishedKeys_whenDownloadedWithNonEmptyStorage_thenReplaceAndAvailableAsMostRecent() {
        assertThat(publicStorage.readPublishedKeys()).isNull()

        runBlocking {
            instance.downloadAndUpdateCachedPublishedKeys()
        }

        assertThat(publicStorage.readPublishedKeys()).isNotNull()
        InstantSubject.assertThat(publicStorage.readPublishedKeysLastUpdate())
            .isCloseTo(clock.now())

        val publishedKeys2 = createUnverifiablePublishedKeys()
        val apiClient2 = CoverDropApiClientMock(
            mockedGetPublishedKeysAndProfiles = { publishedKeys2 }
        )
        val instance2 = ApiResponseCache(apiClient2, publicStorage, configuration, clock)

        runBlocking {
            instance2.downloadAndUpdateCachedPublishedKeys()
        }

        assertThat(publicStorage.readPublishedKeys()).isNotNull()
        InstantSubject.assertThat(publicStorage.readPublishedKeysLastUpdate())
            .isCloseTo(clock.now())
    }

    private fun createUnverifiablePublishedKeys(): PublishedKeysAndProfiles {
        val integrationTestVectors = IntegrationTestVectors(context, TestScenario.Minimal)
        val publishedKeys =
            GsonApiJsonAdapter().parsePublishedPublicKeys(integrationTestVectors.readJson("published_keys"))

        // replace with a bad orgPk so that verification will fail
        val invalidOrgKey = PublishedSignedSigningKey(
            key = ByteArray(ED25519_PUBLIC_KEY_BYTES).hexEncode(),
            certificate = "",
            notValidAfter = Instant.now(),
        )
        return publishedKeys.copy(
            keys = listOf(publishedKeys.keys.single().copy(orgPk = invalidOrgKey))
        )
    }

    @Test
    fun testDeadDrops_whenDownloadedWithEmptyStorage_thenAvailableAsMostRecent() {
        assertThat(publicStorage.readDeadDrops().deadDrops).isEmpty()

        runBlocking {
            instance.downloadAndUpdateNewDeadDrops()
        }

        val expected = runBlocking { apiClient.getDeadDrops(0) }

        assertThat(publicStorage.readDeadDrops()).isEqualTo(expected)
        InstantSubject.assertThat(publicStorage.readPublishedDeadDropsLastUpdate())
            .isCloseTo(clock.now())

        val deadDrops = publicStorage.readDeadDrops().deadDrops
        val oldestTimestamp = deadDrops.minOf { it.createdAt }
        val newestTimestamp = deadDrops.maxOf { it.createdAt }
        val coveredDuration = Duration.between(oldestTimestamp, newestTimestamp)
        assertThat(coveredDuration).isAtMost(configuration.deadDropCacheTTL)
    }

    private fun fakeUserFacingDeadDrop(id: Int, createdAt: Instant) =
        PublishedJournalistToUserDeadDrop(
            id = id,
            createdAt = createdAt,
            data = "",
            cert = "",
            signature = ""
        )

    @Test
    fun testMergeAndTrimDeadDrops_whenDownloadedOverMultipleDateRanges_thenMergedAndTrimmed() {
        val deadDropApril01 = fakeUserFacingDeadDrop(10, Instant.parse("2023-04-01T00:00:00Z"))
        val deadDropApril05 = fakeUserFacingDeadDrop(20, Instant.parse("2023-04-05T00:00:00Z"))
        val deadDropApril06 = fakeUserFacingDeadDrop(21, Instant.parse("2023-04-06T00:00:00Z"))
        val deadDropApril10 = fakeUserFacingDeadDrop(40, Instant.parse("2023-04-10T00:00:00Z"))
        val deadDropApril11 = fakeUserFacingDeadDrop(50, Instant.parse("2023-04-11T00:00:00Z"))
        val deadDropApril20 = fakeUserFacingDeadDrop(80, Instant.parse("2023-04-20T00:00:00Z"))
        val deadDropJune01 = fakeUserFacingDeadDrop(200, Instant.parse("2023-06-01T00:00:00Z"))
        val deadDropJune07 = fakeUserFacingDeadDrop(201, Instant.parse("2023-06-01T00:00:00Z"))

        // Start with an empty storage
        var existingDeadDrops = PublishedJournalistToUserDeadDropsList(
            emptyList()
        )

        // Add dead drops on April 10 that range from April 1 to April 10
        val newDeadDropsApril10 = PublishedJournalistToUserDeadDropsList(
            listOf(deadDropApril01, deadDropApril05, deadDropApril06, deadDropApril10)
        )

        // After merging and trimming we expect that we only have dead drops that range from
        // April 1 to April 10. I.e., all of them
        existingDeadDrops = instance.mergeAndTrimDeadDrops(existingDeadDrops, newDeadDropsApril10)
        assertThat(existingDeadDrops.deadDrops).containsExactly(
            deadDropApril01,
            deadDropApril05,
            deadDropApril06,
            deadDropApril10,
        )

        // Add dead drops on April 20 that range from April 11 to April 20
        val newDeadDropsApril20 = PublishedJournalistToUserDeadDropsList(
            listOf(deadDropApril11, deadDropApril20)
        )

        // After merging and trimming we expect that we only have dead drops that range from
        // April 6 to April 20 (i.e. deadDropCacheTTL).
        existingDeadDrops = instance.mergeAndTrimDeadDrops(existingDeadDrops, newDeadDropsApril20)
        assertThat(existingDeadDrops.deadDrops).containsExactly(
            deadDropApril06,  // just barely in by 1 second because the cut-off-date is inclusive
            deadDropApril10,
            deadDropApril11,
            deadDropApril20,
        )

        // Add dead drops on June 7 that range from June 1 to June 7
        val newDeadDropsJune07 = PublishedJournalistToUserDeadDropsList(
            listOf(deadDropJune01, deadDropJune07)
        )

        // After merging and trimming we expect that we only have dead drops that range from
        // June 1 to June 7.
        existingDeadDrops = instance.mergeAndTrimDeadDrops(existingDeadDrops, newDeadDropsJune07)
        assertThat(existingDeadDrops.deadDrops).containsExactly(
            deadDropJune01,
            deadDropJune07,
        )
    }

    private val testLastDownload = Instant.now()
    private val testMinimumDuration = Duration.ofSeconds(42)

    @Test
    fun testShouldDownload_whenAtLeastMinimumDurationPassed_thenTrue() {
        val actual = instance.shouldDownload(
            now = testLastDownload + testMinimumDuration,
            lastDownload = testLastDownload,
            minimumDurationBetweenDownloads = testMinimumDuration
        )
        assertThat(actual).isTrue()
    }

    @Test
    fun testShouldDownload_whenClockBackwardJump_thenTrue() {
        val actual = instance.shouldDownload(
            now = testLastDownload - Duration.ofSeconds(1),
            lastDownload = testLastDownload,
            minimumDurationBetweenDownloads = testMinimumDuration
        )
        assertThat(actual).isTrue()
    }

    @Test
    fun testShouldDownload_whenNotEnoughTimePassed_thenFalse() {
        val actual = instance.shouldDownload(
            now = testLastDownload + testMinimumDuration.dividedBy(2),
            lastDownload = testLastDownload,
            minimumDurationBetweenDownloads = testMinimumDuration
        )
        assertThat(actual).isFalse()
    }

}
