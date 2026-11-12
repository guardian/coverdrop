package com.theguardian.coverdrop.core.api

import androidx.test.platform.app.InstrumentationRegistry
import com.google.common.truth.Truth.assertThat
import com.theguardian.coverdrop.core.api.models.PublishedJournalistToUserDeadDrop
import com.theguardian.coverdrop.core.api.models.PublishedJournalistToUserDeadDropsList
import com.theguardian.coverdrop.core.api.models.PublishedKeysAndProfiles
import com.theguardian.coverdrop.core.api.models.PublishedSignedSigningKey
import com.theguardian.coverdrop.core.api.models.VerifiedDeadDrop
import com.theguardian.coverdrop.core.crypto.ED25519_PUBLIC_KEY_BYTES
import com.theguardian.coverdrop.core.mocks.CoverDropApiClientMock
import com.theguardian.coverdrop.core.persistence.CoverDropFileManager
import com.theguardian.coverdrop.core.persistence.CoverDropNamespace
import com.theguardian.coverdrop.core.persistence.PublicStorage
import com.theguardian.coverdrop.core.utils.base64Encode
import com.theguardian.coverdrop.core.utils.hexEncode
import com.theguardian.coverdrop.testutils.InstantSubject
import com.theguardian.coverdrop.testutils.IntegrationTestVectors
import com.theguardian.coverdrop.testutils.TestApiCallProvider
import com.theguardian.coverdrop.testutils.TestClock
import com.theguardian.coverdrop.testutils.TestScenario
import com.theguardian.coverdrop.testutils.createMinimalCoverDropTestConfigurationWithClock
import kotlinx.coroutines.runBlocking
import org.junit.After
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
    // The fake dead drops in these tests are not signed, so the test decides which of the stored dead drops count as
    // verified (all of them by default).
    private fun verifiedDeadDrops(isVerified: (PublishedJournalistToUserDeadDrop) -> Boolean = { true }) =
        publicStorage.readDeadDrops().deadDrops.filter(isVerified).map { VerifiedDeadDrop(it.createdAt, emptyList()) }

    private val instance = ApiResponseCache(apiClient, publicStorage, configuration, clock) { verifiedDeadDrops() }

    @Before
    @After
    fun cleanUp() {
        // the storage is shared with other test classes, which must not see our unsigned fake dead drops
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
        val instance2 = ApiResponseCache(apiClient2, publicStorage, configuration, clock) { verifiedDeadDrops() }

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

        // dead drops are stored with whole-second precision, which is what the CoverNode signs
        val expected = instance.normalizeAndDeduplicate(runBlocking { apiClient.getDeadDrops(Instant.EPOCH) })

        assertThat(publicStorage.readDeadDrops()).isEqualTo(expected)
        InstantSubject.assertThat(publicStorage.readPublishedDeadDropsLastUpdate())
            .isCloseTo(clock.now())

        val deadDrops = publicStorage.readDeadDrops().deadDrops
        val oldestTimestamp = deadDrops.minOf { it.createdAt }
        val newestTimestamp = deadDrops.maxOf { it.createdAt }
        val coveredDuration = Duration.between(oldestTimestamp, newestTimestamp)
        assertThat(coveredDuration).isAtMost(configuration.deadDropCacheTTL)
    }

    // The content (data and signature) is derived from the `seed`, so dead drops with different seeds are different
    // dead drops. Use the same `seed` to create copies of the same signed dead drop.
    private fun fakeUserFacingDeadDrop(seed: Int, createdAt: Instant) =
        PublishedJournalistToUserDeadDrop(
            createdAt = createdAt,
            data = "data-$seed".toByteArray().base64Encode(),
            signature = "signature-$seed".toByteArray().hexEncode()
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
        existingDeadDrops = instance.mergeAndTrimDeadDrops(
            existingDeadDrops, newDeadDropsApril10, trimReference = deadDropApril10.createdAt
        )
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
        existingDeadDrops = instance.mergeAndTrimDeadDrops(
            existingDeadDrops, newDeadDropsApril20, trimReference = deadDropApril20.createdAt
        )
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
        existingDeadDrops = instance.mergeAndTrimDeadDrops(
            existingDeadDrops, newDeadDropsJune07, trimReference = deadDropJune07.createdAt
        )
        assertThat(existingDeadDrops.deadDrops).containsExactly(
            deadDropJune01,
            deadDropJune07,
        )
    }

    @Test
    fun testDiscardDeadDropsAtOrBeforeCursor_discardsDeadDropsNotAfterCursor() {
        val cursor = Instant.parse("2023-04-10T00:00:00Z")
        val deadDropBeforeCursor = fakeUserFacingDeadDrop(1, Instant.parse("2023-04-09T00:00:00Z"))
        val deadDropAtCursor = fakeUserFacingDeadDrop(2, cursor)
        val deadDropAfterCursor = fakeUserFacingDeadDrop(3, Instant.parse("2023-04-11T00:00:00Z"))

        val filtered = instance.filterKeepOnlyDeadDropsAfterCursor(
            deadDrops = PublishedJournalistToUserDeadDropsList(
                listOf(deadDropBeforeCursor, deadDropAtCursor, deadDropAfterCursor)
            ),
            cursor = cursor,
        )

        assertThat(filtered.deadDrops).containsExactly(deadDropAfterCursor)
    }

    @Test
    fun testDeadDrops_whenApiReplaysDeadDropsNotAfterCursor_thenDiscardedAndStorageUnchanged() {
        val existingDeadDrop = fakeUserFacingDeadDrop(1, Instant.parse("2023-04-10T00:00:00Z"))
        val existing = PublishedJournalistToUserDeadDropsList(listOf(existingDeadDrop))
        publicStorage.writeDeadDrops(existing)

        // a misbehaving API ignores the cursor and replays two dead drops, one before and one at the cursor
        val replayedBefore = fakeUserFacingDeadDrop(2, Instant.parse("2023-04-09T00:00:00Z"))
        val replayedAtCursor = fakeUserFacingDeadDrop(3, existingDeadDrop.createdAt)
        val replayingApiClient = CoverDropApiClientMock(
            mockedGetDeadDrops = {
                PublishedJournalistToUserDeadDropsList(listOf(replayedBefore, replayedAtCursor))
            }
        )
        val replayingInstance = ApiResponseCache(replayingApiClient, publicStorage, configuration, clock) {
            verifiedDeadDrops()
        }

        runBlocking {
            replayingInstance.downloadAndUpdateNewDeadDrops()
        }

        assertThat(publicStorage.readDeadDrops()).isEqualTo(existing)
    }

    @Test
    fun testNormalizeAndDeduplicate_truncatesAndMergesCopiesOfSameSignedDeadDrop() {
        val signedSecond = Instant.parse("2023-04-10T12:34:56Z")
        val original = fakeUserFacingDeadDrop(7, signedSecond)

        // a copy with only the unsigned sub-second part of `createdAt` differing
        val copyWithSubSecond = original.copy(createdAt = signedSecond.plusNanos(389_000_000))

        // different signed content: whole second, data, and tampered data under the same signature
        val nextSecond = fakeUserFacingDeadDrop(7, signedSecond.plusSeconds(1))
        val otherData = fakeUserFacingDeadDrop(8, signedSecond)
        val tamperedData = original.copy(data = "tampered".toByteArray().base64Encode())

        val normalized = instance.normalizeAndDeduplicate(
            PublishedJournalistToUserDeadDropsList(
                listOf(original, copyWithSubSecond, nextSecond, otherData, tamperedData)
            )
        )

        assertThat(normalized.deadDrops).containsExactly(original, nextSecond, otherData, tamperedData)
    }

    @Test
    fun testDeadDrops_whenApiServesCursorDeadDropAgainWithSubSecondPrecision_thenStorageUnchanged() {
        // The API returns dead drops with `created_at > cursor` and its `created_at` has sub-second precision.
        // Because the cursor is truncated to whole seconds, the dead drop that the cursor was taken from is served
        // again. After truncating it is equal to the cursor, so it is discarded.
        val cursorSecond = Instant.parse("2023-04-10T00:00:00Z")
        val existing = PublishedJournalistToUserDeadDropsList(listOf(fakeUserFacingDeadDrop(1, cursorSecond)))
        publicStorage.writeDeadDrops(existing)

        val servedAgain = fakeUserFacingDeadDrop(1, cursorSecond.plusNanos(389_000_000))
        val servedAgainResponse = PublishedJournalistToUserDeadDropsList(listOf(servedAgain))
        val servingAgainInstance = ApiResponseCache(
            CoverDropApiClientMock(mockedGetDeadDrops = { servedAgainResponse }),
            publicStorage, configuration, clock
        ) { verifiedDeadDrops() }

        runBlocking { servingAgainInstance.downloadAndUpdateNewDeadDrops() }

        assertThat(publicStorage.readDeadDrops()).isEqualTo(existing)
    }

    @Test
    fun testDeadDrops_whenApiReplaysNewDeadDropWithDifferentTimestamps_thenStoredOnlyOnce() {
        val cursorSecond = Instant.parse("2023-04-10T00:00:00Z")
        publicStorage.writeDeadDrops(
            PublishedJournalistToUserDeadDropsList(listOf(fakeUserFacingDeadDrop(1, cursorSecond)))
        )

        // a malicious API serves one genuinely signed new dead drop many times with different sub-second timestamps
        val replayedCopies = (0 until 1000).map {
            fakeUserFacingDeadDrop(2, cursorSecond.plusSeconds(1).plusNanos(it * 1_000L))
        }
        val replayingInstance = ApiResponseCache(
            CoverDropApiClientMock(mockedGetDeadDrops = { PublishedJournalistToUserDeadDropsList(replayedCopies) }),
            publicStorage, configuration, clock
        ) { verifiedDeadDrops() }

        runBlocking { replayingInstance.downloadAndUpdateNewDeadDrops() }

        // the existing dead drop and one copy of the new dead drop
        assertThat(publicStorage.readDeadDrops().deadDrops).hasSize(2)
    }

    @Test
    fun testDeadDrops_whenVerifiedKeysNotAvailable_thenDoesNotDownload() {
        val existing = PublishedJournalistToUserDeadDropsList(
            listOf(fakeUserFacingDeadDrop(1, Instant.parse("2023-04-10T00:00:00Z")))
        )
        publicStorage.writeDeadDrops(existing)

        // the API mock fails the test if it is called
        val instanceWithoutKeys = ApiResponseCache(
            CoverDropApiClientMock(), publicStorage, configuration, clock
        ) { error("no verified keys") }

        runBlocking { instanceWithoutKeys.downloadAndUpdateNewDeadDrops() }

        assertThat(publicStorage.readDeadDrops()).isEqualTo(existing)
    }

    @Test
    fun testDeadDrops_whenForgedFarFutureDeadDropStored_thenGenuineDeadDropsAreNotEvicted() {
        val t0 = Instant.parse("2023-04-10T00:00:00Z")
        val genuine = fakeUserFacingDeadDrop(1, t0)
        // a malicious API served a dead drop that does not verify and whose `createdAt` is far in the future
        val forged = fakeUserFacingDeadDrop(2, t0.plus(Duration.ofDays(100)))
        publicStorage.writeDeadDrops(PublishedJournalistToUserDeadDropsList(listOf(genuine, forged)))

        val newGenuine = fakeUserFacingDeadDrop(3, t0.plus(Duration.ofHours(1)))
        val servingInstance = ApiResponseCache(
            CoverDropApiClientMock(mockedGetDeadDrops = { PublishedJournalistToUserDeadDropsList(listOf(newGenuine)) }),
            publicStorage, configuration, clock
        ) { verifiedDeadDrops { it != forged } }

        runBlocking { servingInstance.downloadAndUpdateNewDeadDrops() }

        // the forged dead drop must not be used as the reference for "now" when trimming
        assertThat(publicStorage.readDeadDrops().deadDrops).containsAtLeast(genuine, newGenuine)
        // the forged dead drop ends up in public storage because the cursor is verified but the dead drops are not.
        assertThat(publicStorage.readDeadDrops().deadDrops).contains(forged)
    }

    @Test
    fun testDeadDrops_cursorIgnoresStoredDeadDropsThatDoNotVerify() {
        val verifiedCreatedAt = Instant.parse("2023-04-10T00:00:00Z")
        val verified = fakeUserFacingDeadDrop(1, verifiedCreatedAt)
        // a malicious API served a dead drop with a made-up `createdAt` that does not verify
        val forged = fakeUserFacingDeadDrop(2, verifiedCreatedAt.plus(Duration.ofDays(100)))
        publicStorage.writeDeadDrops(PublishedJournalistToUserDeadDropsList(listOf(verified, forged)))

        var requestedCursor: Instant? = null
        val trackingInstance = ApiResponseCache(
            CoverDropApiClientMock(mockedGetDeadDrops = {
                requestedCursor = it
                PublishedJournalistToUserDeadDropsList(emptyList())
            }),
            publicStorage, configuration, clock
        ) { verifiedDeadDrops { it != forged } }

        runBlocking { trackingInstance.downloadAndUpdateNewDeadDrops() }

        assertThat(requestedCursor).isEqualTo(verifiedCreatedAt)
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
