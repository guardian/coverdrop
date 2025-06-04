package com.theguardian.coverdrop.core.api

import com.theguardian.coverdrop.core.ICoverDropLibInternal
import com.theguardian.coverdrop.core.api.models.PublishedJournalistToUserDeadDropsList
import com.theguardian.coverdrop.core.api.models.PublishedKeysAndProfiles
import com.theguardian.coverdrop.core.api.models.PublishedStatusEvent
import com.theguardian.coverdrop.core.api.models.UserMessage


data class ApiConfiguration(
    val apiBaseUrl: String,
    val messagingBaseUrl: String,
) {
    init {
        require(!apiBaseUrl.endsWith("/")) { "The base URL must not have a trailing '/'" }
        require(!messagingBaseUrl.endsWith("/")) { "The messaging base URL must not have a trailing '/'" }
    }
}

class ApiCallProviderException(message: String, cause: Throwable?) : Exception(message, cause)

/**
 * The [IApiCallProvider] is to be implemented by the integrating application. This is to allow
 * reusing the existing HTTP client library and (where possible) re-use connections to the same
 * endpoint.
 */
interface IApiCallProvider {

    /**
     * Performs a GET request for the [ApiConfiguration.apiBaseUrl] and [path] with the given URL
     * [queryParameters] (if any). The response is expected to be a JSON [String] which is then
     * returned.
     *
     * In case of errors, a [ApiCallProviderException] should be thrown. These are the only ones
     * that are handled. All others will propagate to the top-most level.
     */
    @Throws(ApiCallProviderException::class)
    suspend fun getJsonApi(
        apiConfiguration: ApiConfiguration,
        path: String,
        queryParameters: List<Pair<String, String>>? = null,
    ): String

    /**
     * Performs a POST request for the [ApiConfiguration.messagingBaseUrl] and [path]. The body
     * payload is must be the provided [json] [String] and the mimetype must be `application/json`.
     *
     * In case of errors, a [ApiCallProviderException] should be thrown. These are the only ones
     * that are handled. All others will propagate to the top-most level.
     */
    @Throws(ApiCallProviderException::class)
    suspend fun postJsonMessaging(
        apiConfiguration: ApiConfiguration,
        path: String,
        json: String,
    )
}

internal interface ICoverDropApiClient {

    @Throws(ApiCallProviderException::class)
    suspend fun getPublishedStatusEvent(): PublishedStatusEvent

    @Throws(ApiCallProviderException::class)
    suspend fun getPublishedKeys(): PublishedKeysAndProfiles

    @Throws(ApiCallProviderException::class)
    suspend fun getDeadDrops(idsGreaterThan: Int = 0): PublishedJournalistToUserDeadDropsList

    @Throws(ApiCallProviderException::class)
    suspend fun postMessage(message: UserMessage)
}

internal class CoverDropApiClient(coverDropLib: ICoverDropLibInternal) : ICoverDropApiClient {

    private val apiConfiguration = coverDropLib.getConfig().apiConfiguration
    private val apiCallProvider = coverDropLib.getApiCallProvider()
    private val apiJsonAdapter = GsonApiJsonAdapter()

    override suspend fun getPublishedStatusEvent(): PublishedStatusEvent {
        val json = apiCallProvider.getJsonApi(apiConfiguration, "/v1/status")
        return apiJsonAdapter.parsePublishedStatusEvent(json)
    }

    override suspend fun getPublishedKeys(): PublishedKeysAndProfiles {
        val json = apiCallProvider.getJsonApi(apiConfiguration, "/v1/public-keys")
        return apiJsonAdapter.parsePublishedPublicKeys(json)
    }

    override suspend fun getDeadDrops(idsGreaterThan: Int): PublishedJournalistToUserDeadDropsList {
        require(idsGreaterThan >= 0)
        val params = listOf(Pair("ids_greater_than", idsGreaterThan.toString()))

        val json = apiCallProvider.getJsonApi(apiConfiguration, "/v1/user/dead-drops", params)
        return apiJsonAdapter.parsePublishedDeadDrops(json)
    }

    override suspend fun postMessage(message: UserMessage) {
        val json = apiJsonAdapter.jsonifyUserMessage(message)
        // The result of `jsonifyUserMessage` is not valid per RFC 4627 since it is simple a string
        // on the top-level. The message format is checked on the CDN entry anyway.
        apiCallProvider.postJsonMessaging(apiConfiguration, "/user/messages", json)
    }
}
