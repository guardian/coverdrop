package com.theguardian.coverdrop.core.api

import android.util.Log
import androidx.annotation.VisibleForTesting
import com.theguardian.coverdrop.core.CoverDropConfiguration
import com.theguardian.coverdrop.core.api.models.PublishedJournalistToUserDeadDropsList
import com.theguardian.coverdrop.core.api.models.PublishedKeysAndProfiles
import com.theguardian.coverdrop.core.api.models.VerifiedDeadDrops
import com.theguardian.coverdrop.core.persistence.PublicStorage
import com.theguardian.coverdrop.core.utils.IClock
import java.time.Duration
import java.time.Instant
import java.time.temporal.ChronoUnit

/**
 * The [ApiResponseCache] is responsible for maintaining the recent versions of [StatusEvent],
 * [PublishedKeysAndProfiles] and [PublishedJournalistToUserDeadDropsList] on the device.
 * For this the [downloadAllUpdates] method is to be called on app start.
 */
internal class ApiResponseCache(
    private val apiClient: ICoverDropApiClient,
    private val publicStorage: PublicStorage,
    private val configuration: CoverDropConfiguration,
    private val clock: IClock = configuration.clock,
    private val getVerifiedDeadDrops: suspend () -> VerifiedDeadDrops,
) {
    suspend fun downloadAllUpdates(force: Boolean = false) {
        internalExecute(
            lastDownload = publicStorage.readPublishedStatusEventLastUpdate(),
            downloadAction = { downloadAndUpdateStatusEvent() },
            cachedFileExists = publicStorage.hasPublishedStatusEvent(),
            minimumDurationBetweenDownloads = configuration.minimumDurationBetweenStatusUpdateDownloads,
            force = force,
        )
        internalExecute(
            lastDownload = publicStorage.readPublishedKeysLastUpdate(),
            downloadAction = { downloadAndUpdateCachedPublishedKeys() },
            cachedFileExists = publicStorage.hasPublishedKeys(),
            minimumDurationBetweenDownloads = configuration.minimumDurationBetweenDefaultDownloads,
            force = force,
        )
        internalExecute(
            lastDownload = publicStorage.readPublishedDeadDropsLastUpdate(),
            downloadAction = { downloadAndUpdateNewDeadDrops() },
            cachedFileExists = publicStorage.hasDeadDrops(),
            minimumDurationBetweenDownloads = configuration.minimumDurationBetweenDefaultDownloads,
            force = force,
        )
    }

    private suspend fun internalExecute(
        lastDownload: Instant?,
        downloadAction: suspend () -> Unit,
        cachedFileExists: Boolean,
        minimumDurationBetweenDownloads: Duration,
        force: Boolean = false,
    ) {
        // if we have a known last download time and the file exists, check if enough time passed
        if (lastDownload != null && cachedFileExists) {
            val skipDownload = !shouldDownload(
                now = clock.now(),
                lastDownload = lastDownload,
                minimumDurationBetweenDownloads = minimumDurationBetweenDownloads,
            )

            // skip if too recent and not forced
            if (skipDownload && !force) {
                return
            }
        }

        // otherwise, go ahead
        downloadAction()
    }

    @VisibleForTesting(otherwise = VisibleForTesting.PRIVATE)
    @Throws(ApiCallProviderException::class)
    internal suspend fun downloadAndUpdateStatusEvent() {
        try {
            val publishedStatusEvent = apiClient.getPublishedStatusEvent()
            publicStorage.writePublishedStatusEvent(publishedStatusEvent)
            publicStorage.writePublishedStatusEventLastUpdate(clock.now())
        } catch (e: ApiCallProviderException) {
            if (configuration.localTestMode) {
                // This log statement is safe because it is test-mode only and independent of usage
                Log.d("ApiResponseCache", "failed to download status event: $e")
            }
        }
    }

    @VisibleForTesting(otherwise = VisibleForTesting.PRIVATE)
    @Throws(ApiCallProviderException::class)
    internal suspend fun downloadAndUpdateCachedPublishedKeys() {
        try {
            val publishedKeys = apiClient.getPublishedKeys()
            publicStorage.writePublishedKeys(publishedKeys)
            publicStorage.writePublishedKeysLastUpdate(clock.now())
        } catch (e: ApiCallProviderException) {
            if (configuration.localTestMode) {
                // This log statement is safe because it is test-mode only and independent of usage
                Log.d("ApiResponseCache", "failed to download published keys: $e")
            }
        }
    }

    @VisibleForTesting(otherwise = VisibleForTesting.PRIVATE)
    @Throws(ApiCallProviderException::class)
    internal suspend fun downloadAndUpdateNewDeadDrops() {
        // dead-drops cached by earlier app versions can carry sub-second precision or duplicates
        val existingDeadDrops = normalizeAndDeduplicate(publicStorage.readDeadDrops())

        // without verified keys we cannot establish a trustworthy cursor, so we do not download
        val verifiedDeadDrops = runCatching { getVerifiedDeadDrops() }.getOrNull() ?: return

        // the cache can contain dead-drops that do not verify, e.g. ones with a made-up `createdAt`
        // served by a malicious API; only the verified ones are allowed to move the cursor
        val largestExistingCreatedAt = verifiedDeadDrops.maxOfOrNull { it.createdAt }
            ?.truncatedTo(ChronoUnit.SECONDS) ?: Instant.EPOCH

        val publishedDeadDrops = try {
            apiClient.getDeadDrops(createdAfter = largestExistingCreatedAt)
        } catch (e: ApiCallProviderException) {
            if (configuration.localTestMode) {
                // This log statement is safe because it is test-mode only and independent of usage
                Log.d("ApiResponseCache", "failed to download new dead-drops: $e")
            }
            return
        }

        // a malicious or buggy API could replay dead drops we've already processed by serving
        // them again at or before the requested cursor, so discard any that don't satisfy
        // `createdAt > cursor`
        val filteredPublishedDeadDrops = filterKeepOnlyDeadDropsAfterCursor(
            deadDrops = normalizeAndDeduplicate(publishedDeadDrops),
            cursor = largestExistingCreatedAt,
        )

        // if there are none, we can finish early
        if (filteredPublishedDeadDrops.deadDrops.isEmpty()) {
            return
        }

        // merge the existing and new dead-drops; this involves combining them and then removing
        // those that are older than the set cache TTL based on the newest verified timestamp
        val mergedTrimmedDeadDropList = mergeAndTrimDeadDrops(
            existingDeadDrops = existingDeadDrops,
            newDeadDrops = filteredPublishedDeadDrops,
            trimReference = largestExistingCreatedAt,
        )

        // write back to disk
        publicStorage.writeDeadDrops(mergedTrimmedDeadDropList)
        publicStorage.writePublishedDeadDropsUpdate(clock.now())
    }

    /**
     * Truncates every `createdAt` to whole seconds, the precision that is signed by the CoverNode,
     * and removes duplicates. The sub-second part is not signed, so without this a malicious API
     * could serve one signed dead-drop many times with different timestamps, and every copy would
     * be treated as a distinct dead-drop. The first occurrence of a duplicate is kept.
     */
    @VisibleForTesting(otherwise = VisibleForTesting.PRIVATE)
    internal fun normalizeAndDeduplicate(
        deadDrops: PublishedJournalistToUserDeadDropsList,
    ): PublishedJournalistToUserDeadDropsList {
        val normalized = deadDrops.deadDrops.map { it.copy(createdAt = it.createdAt.truncatedTo(ChronoUnit.SECONDS)) }
        return PublishedJournalistToUserDeadDropsList(normalized.distinct())
    }

    @VisibleForTesting(otherwise = VisibleForTesting.PRIVATE)
    internal fun filterKeepOnlyDeadDropsAfterCursor(
        deadDrops: PublishedJournalistToUserDeadDropsList,
        cursor: Instant,
    ): PublishedJournalistToUserDeadDropsList {
        val filteredDeadDrops = deadDrops.deadDrops.filter { deadDrop ->
            deadDrop.createdAt > cursor
        }

        return PublishedJournalistToUserDeadDropsList(filteredDeadDrops)
    }

    @VisibleForTesting(otherwise = VisibleForTesting.PRIVATE)
    internal fun mergeAndTrimDeadDrops(
        existingDeadDrops: PublishedJournalistToUserDeadDropsList,
        newDeadDrops: PublishedJournalistToUserDeadDropsList,
        trimReference: Instant,
    ): PublishedJournalistToUserDeadDropsList {
        // merge both lists
        val mergedDeadDrops = normalizeAndDeduplicate(
            PublishedJournalistToUserDeadDropsList(existingDeadDrops.deadDrops + newDeadDrops.deadDrops)
        ).deadDrops.sortedBy { it.createdAt }

        // We use the newest verified dead-drop timestamp as a reference for "now" to avoid using the device clock
        // which might be out of sync leading to the eviction of more or fewer items than intended. The proxy for "now"
        // must come from a verified dead drop, otherwise a malicious API could serve one with a far-future
        // `createdAt` and make the client evict all genuine dead drops.
        val cutOffDate = trimReference - configuration.deadDropCacheTTL
        val mergedAndTrimmedDeadDrops = mergedDeadDrops.filter { it.createdAt >= cutOffDate }

        return PublishedJournalistToUserDeadDropsList(mergedAndTrimmedDeadDrops)
    }

    /**
     * Returns `true` if the minimum duration has passed or we detect some general clock mishap.
     */
    @VisibleForTesting(otherwise = VisibleForTesting.PRIVATE)
    internal fun shouldDownload(
        now: Instant,
        lastDownload: Instant,
        minimumDurationBetweenDownloads: Duration,
    ): Boolean {
        // if the last download appears to be in the future, the device clock has jumped backwards;
        // in this case we should download (which then updates our `lastDownload` timestamp)
        if (lastDownload > now) return true

        // if at least the minimum duration has passed, we should download
        if (lastDownload + minimumDurationBetweenDownloads <= now) return true

        // otherwise, we hold off
        return false
    }
}
