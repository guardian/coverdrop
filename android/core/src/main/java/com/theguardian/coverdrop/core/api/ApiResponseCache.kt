package com.theguardian.coverdrop.core.api

import android.util.Log
import androidx.annotation.VisibleForTesting
import com.theguardian.coverdrop.core.CoverDropConfiguration
import com.theguardian.coverdrop.core.api.models.PublishedJournalistToUserDeadDropsList
import com.theguardian.coverdrop.core.api.models.PublishedKeysAndProfiles
import com.theguardian.coverdrop.core.persistence.PublicStorage
import com.theguardian.coverdrop.core.utils.IClock
import java.time.Duration
import java.time.Instant

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
        val existingDeadDrops = publicStorage.readDeadDrops()

        // download new dead-drops
        val largestExistingId = existingDeadDrops.deadDrops.maxOfOrNull { it.id }

        val publishedDeadDrops = try {
            apiClient.getDeadDrops(idsGreaterThan = largestExistingId ?: 0)
        } catch (e: ApiCallProviderException) {
            if (configuration.localTestMode) {
                // This log statement is safe because it is test-mode only and independent of usage
                Log.d("ApiResponseCache", "failed to download new dead-drops: $e")
            }
            return
        }

        // if there are none, we can finish early
        if (publishedDeadDrops.deadDrops.isEmpty()) {
            return
        }

        // merge the existing and new dead-drops; this involves combining them and then removing
        // those who are older than the set cache TTL based on the most-recent timestamp in the
        // combined set
        val mergedTrimmedDeadDropList = mergeAndTrimDeadDrops(existingDeadDrops, publishedDeadDrops)

        // write back to disk
        publicStorage.writeDeadDrops(mergedTrimmedDeadDropList)
        publicStorage.writePublishedDeadDropsUpdate(clock.now())
    }

    @VisibleForTesting(otherwise = VisibleForTesting.PRIVATE)
    internal fun mergeAndTrimDeadDrops(
        existingDeadDrops: PublishedJournalistToUserDeadDropsList,
        newDeadDrops: PublishedJournalistToUserDeadDropsList
    ): PublishedJournalistToUserDeadDropsList {
        // merge both lists
        val mergedDeadDrops = (existingDeadDrops.deadDrops + newDeadDrops.deadDrops)
            .sortedBy { it.id }
            .toList()

        // identify the newest dead-drop timestamp. We use that as a reference for "now" to avoid
        // using the device clock which might be out-of-sync and could lead to evicting more of
        // fewer items than intended
        val mostRecentTimestamp = mergedDeadDrops.maxOf { it.createdAt }
        val cutOffDate = mostRecentTimestamp - configuration.deadDropCacheTTL
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
