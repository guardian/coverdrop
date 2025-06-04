package com.theguardian.coverdrop.core

import android.util.Log
import com.theguardian.coverdrop.core.api.ApiCallProviderException
import com.theguardian.coverdrop.core.api.ApiResponseCache
import com.theguardian.coverdrop.core.api.CoverDropApiClient
import com.theguardian.coverdrop.core.api.models.PublishedJournalistProfile
import com.theguardian.coverdrop.core.api.models.PublishedJournalistVisibility
import com.theguardian.coverdrop.core.api.models.PublishedKeysAndProfiles
import com.theguardian.coverdrop.core.api.models.PublishedStatusEvent
import com.theguardian.coverdrop.core.api.models.UserMessage
import com.theguardian.coverdrop.core.api.models.VerifiedDeadDrops
import com.theguardian.coverdrop.core.api.models.VerifiedKeys
import com.theguardian.coverdrop.core.api.models.mostRecentMessagingKeyForEachCoverNode
import com.theguardian.coverdrop.core.crypto.CoverDropPrivateSendingQueue
import com.theguardian.coverdrop.core.crypto.PrivateSendingQueueHint
import com.theguardian.coverdrop.core.crypto.PrivateSendingQueueItem
import com.theguardian.coverdrop.core.crypto.PrivateSendingQueueSecret
import com.theguardian.coverdrop.core.crypto.PublicSigningKey
import com.theguardian.coverdrop.core.models.JournalistId
import com.theguardian.coverdrop.core.models.JournalistInfo
import com.theguardian.coverdrop.core.models.JournalistTag
import com.theguardian.coverdrop.core.models.JournalistVisibility
import com.theguardian.coverdrop.core.models.StatusEvent
import com.theguardian.coverdrop.core.utils.base64Encode
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

interface ICoverDropPublicDataRepository {
    /**
     * Returns the list of all recipients that are available including both desks and individual
     * journalists. This list is not verified.
     */
    suspend fun getAllJournalists(includeHidden: Boolean = false): List<JournalistInfo>

    /**
     * Returns the default journalist info (if any)
     */
    suspend fun getDefaultJournalist(): JournalistInfo?

    /**
     * Returns the most current [StatusEvent].
     *
     * @throws [IllegalStateException] if no keys are available.
     */
    suspend fun getStatusEvent(): StatusEvent
}

internal interface ICoverDropPublicDataRepositoryInternal : ICoverDropPublicDataRepository {
    /**
     * Initialize the repository by updating the cached API responses (if required) and
     * running [PrivateSendingQueueProvider.initialize]. This is called on app start by the
     * CoverDrop library.
     */
    suspend fun initialize()

    /**
     * Forces a download of updates from the server. The app should not call this directly as it
     * is already called within the initialization phase at app start. It is only left here to allow
     * for manual testing --- as such it will throw if called outside the local test mode.
     */
    suspend fun forceUpdateCachedApiResponses()

    /**
     * Updates the cached API responses if the last update has been far enough in the past. Hence,
     * it is safe to optimistically call this method on app foreground events and similar.
     */
    suspend fun maybeUpdateCachedApiResponses()

    /**
     * Returns the most-recent verified keys cached on the device.
     *
     * @throws [IllegalStateException] if no keys are available.
     * @throws [KeyVerificationException] if the keys are not valid.
     */
    suspend fun getVerifiedKeys(): VerifiedKeys

    /**
     * Returns the [JournalistTag] for the given [JournalistId] (if any).
     */
    suspend fun getJournalistTag(id: JournalistId): JournalistTag?

    /**
     * Returns the most-recent verified dead drops that are cached on the device. Might return an
     * empty list if no dead drops are available.
     */
    suspend fun getMostRecentDeadDrops(): VerifiedDeadDrops

    /**
     * Adds a message to the [CoverDropPrivateSendingQueue] that will be sent to the server by the
     * background worker. The message must be encrypted using
     * [Protocol.encryptUserToJournalistMessageViaCoverNode].
     */
    suspend fun addMessageToQueue(
        privateSendingQueueSecret: PrivateSendingQueueSecret,
        message: ByteArray,
    ): PrivateSendingQueueHint

    /**
     * Sends the next message from the [CoverDropPrivateSendingQueue]. If the send operation
     * succeeds the message will be removed from the queue. If the send operation fails the message
     * will remain in the queue.
     */
    suspend fun sendNextMessageFromQueue()

    /**
     * Returns the list of hints for messages that are currently in the queue to be sent.
     */
    suspend fun getPrivateSendingQueueHintsInQueue(): List<PrivateSendingQueueHint>

    /**
     * Removes all real messages from the queue and fills the queue with only cover messages.
     */
    suspend fun clearQueue(secret: PrivateSendingQueueSecret)
}

/**
 * Repository containing all public data available from the API.
 */
internal class CoverDropPublicDataRepository internal constructor(
    lib: ICoverDropLibInternal,
) : ICoverDropPublicDataRepositoryInternal, ICoverMessageFactory {
    private val configuration = lib.getConfig()
    private val publicStorage = lib.getPublicStorage()

    private val apiService = CoverDropApiClient(lib)
    private val apiResponseCache = ApiResponseCache(
        apiClient = apiService,
        publicStorage = publicStorage,
        configuration = configuration
    )
    private val clock = lib.getClock()
    private val configurationTrustedOrgPks = configuration.trustedOrgPks
    private val deadDropParser = lib.getDeadDropParser()
    private val keyVerifier = lib.getKeyVerifier()
    private val privateSendingQueue = lib.getPrivateSendingQueueProvider()
    private val protocol = lib.getProtocol()

    private var localStatusEventOverride: StatusEvent? = null

    override suspend fun initialize() {
        withContext(Dispatchers.Default) {
            publicStorage.initialize()
            apiResponseCache.downloadAllUpdates()
            privateSendingQueue.initialize(this@CoverDropPublicDataRepository)
        }
    }

    override suspend fun getAllJournalists(includeHidden: Boolean): List<JournalistInfo> {
        val journalistList = internalReadPublishedKeys()
            ?.journalistProfiles
            ?.map { entry -> entry.toJournalistInfo() }
            ?.filter { includeHidden || it.visibility == JournalistVisibility.VISIBLE }

        return journalistList ?: emptyList()
    }

    override suspend fun getDefaultJournalist(): JournalistInfo? {
        val defaultId = internalReadPublishedKeys()?.defaultJournalistId
        return getAllJournalists().find { it.id == defaultId }
    }

    override suspend fun getStatusEvent(): StatusEvent {
        localStatusEventOverride?.run { return this }

        return withContext(Dispatchers.Default) {
            val cachedStatusEvent = requireNotNull(internalReadPublishedStatusEvent()) {
                "No status event cached"
            }
            StatusEvent(
                status = cachedStatusEvent.getStatus(),
                isAvailable = cachedStatusEvent.isAvailable,
                description = cachedStatusEvent.description
            )
        }
    }

    override suspend fun forceUpdateCachedApiResponses() {
        require(configuration.localTestMode) { "This method must only be called in local test mode" }
        withContext(Dispatchers.Default) {
            apiResponseCache.downloadAllUpdates(force = true)
        }
    }

    override suspend fun maybeUpdateCachedApiResponses() {
        withContext(Dispatchers.Default) {
            apiResponseCache.downloadAllUpdates(force = false)
        }
    }

    override suspend fun getVerifiedKeys(): VerifiedKeys {
        return withContext(Dispatchers.Default) {
            val cachedPublishedKeys = requireNotNull(internalReadPublishedKeys()) {
                "No published keys cached"
            }

            keyVerifier.verifyPublishedKeysAndProfiles(
                publishedKeysAndProfiles = cachedPublishedKeys,
                trustedOrgPks = trustedOrgPks(),
                now = clock.now()
            )
        }
    }

    override suspend fun getJournalistTag(id: JournalistId): JournalistTag? {
        return internalReadPublishedKeys()
            ?.journalistProfiles
            ?.find { it.id == id }
            ?.tag
    }

    override suspend fun getMostRecentDeadDrops(): VerifiedDeadDrops {
        return withContext(Dispatchers.Default) {
            val cachedDeadDrops = publicStorage.readDeadDrops()
            val verifiedKeys = getVerifiedKeys()

            deadDropParser.verifyAndParseDeadDropsList(
                candidate = cachedDeadDrops,
                coverNodeKeyHierarchies = verifiedKeys.keys.flatMap { it.coverNodeHierarchies }
            )
        }
    }

    override suspend fun addMessageToQueue(
        privateSendingQueueSecret: PrivateSendingQueueSecret,
        message: ByteArray,
    ): PrivateSendingQueueHint {
        return withContext(Dispatchers.Default) {
            return@withContext privateSendingQueue.enqueue(
                secret = privateSendingQueueSecret,
                item = PrivateSendingQueueItem(message)
            )
        }
    }

    override suspend fun sendNextMessageFromQueue() {
        return withContext(Dispatchers.Default) {
            val message = privateSendingQueue.peek()
            try {
                apiService.postMessage(UserMessage(message.bytes.base64Encode()))
            } catch (e: ApiCallProviderException) {
                if (configuration.localTestMode) {
                    // This log statement is safe because it is test-mode only and independent of usage
                    Log.d("ApiResponseCache", "failed to send from queue: $e")
                }
            }

            // `postMessage` did not throw which indicates success; hence we can remove the
            // message and persist this new state
            privateSendingQueue.dequeue(this@CoverDropPublicDataRepository)
        }
    }

    override suspend fun getPrivateSendingQueueHintsInQueue(): List<PrivateSendingQueueHint> {
        return withContext(Dispatchers.Default) {
            return@withContext privateSendingQueue.allHints()
        }
    }

    override suspend fun clearQueue(secret: PrivateSendingQueueSecret) {
        withContext(Dispatchers.Default) {
            privateSendingQueue.clear(secret, this@CoverDropPublicDataRepository)
        }
    }

    override suspend fun createCoverMessage(): PrivateSendingQueueItem {
        val verifiedKeys = getVerifiedKeys()

        // Find the most recent key for each coverNode
        val coverNodesToMostRecentKey =
            verifiedKeys.mostRecentMessagingKeyForEachCoverNode(clock)
        require(coverNodesToMostRecentKey.isNotEmpty()) { "No valid covernode key candidate found at all" }

        val coverMessage = protocol.createCoverMessageToCoverNode(coverNodesToMostRecentKey)
        return PrivateSendingQueueItem(coverMessage)
    }

    /**
     * Temporarily override the status event for local errors that appear during initialization.
     */
    internal fun overrideStatusEventForTesting(statusEvent: StatusEvent) {
        require(statusEvent.isAvailable.not()) { "The override should only be used with negative status events." }
        localStatusEventOverride = statusEvent
    }

    private suspend fun internalReadPublishedKeys(): PublishedKeysAndProfiles? {
        return withContext(Dispatchers.Default) {
            publicStorage.readPublishedKeys()
        }
    }

    private suspend fun internalReadPublishedStatusEvent(): PublishedStatusEvent? {
        return withContext(Dispatchers.Default) {
            publicStorage.readPublishedStatusEvent()
        }
    }

    private fun trustedOrgPks(): List<PublicSigningKey> {
        return configurationTrustedOrgPks
    }
}

internal fun PublishedJournalistProfile.toJournalistInfo(): JournalistInfo {
    val visibility = when (status) {
        PublishedJournalistVisibility.VISIBLE -> JournalistVisibility.VISIBLE
        PublishedJournalistVisibility.HIDDEN_FROM_UI -> JournalistVisibility.HIDDEN
        PublishedJournalistVisibility.HIDDEN_FROM_RESPONSE -> JournalistVisibility.HIDDEN
    }
    return JournalistInfo(
        id = id,
        displayName = displayName,
        sortName = sortName,
        description = description,
        isTeam = isDesk,
        tag = tag,
        visibility = visibility
    )
}
