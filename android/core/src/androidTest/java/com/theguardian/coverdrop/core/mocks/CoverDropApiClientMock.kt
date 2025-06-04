package com.theguardian.coverdrop.core.mocks

import com.theguardian.coverdrop.core.api.ApiConfiguration
import com.theguardian.coverdrop.core.api.GsonApiJsonAdapter
import com.theguardian.coverdrop.core.api.ICoverDropApiClient
import com.theguardian.coverdrop.core.api.models.PublishedJournalistToUserDeadDropsList
import com.theguardian.coverdrop.core.api.models.PublishedKeysAndProfiles
import com.theguardian.coverdrop.core.api.models.PublishedStatusEvent
import com.theguardian.coverdrop.core.api.models.UserMessage
import com.theguardian.coverdrop.testutils.TestApiCallProvider
import java.time.Instant

internal class CoverDropApiClientMock(
    private val mockedGetStatusEvent: suspend () -> PublishedStatusEvent = {
        TODO("not mocked")
    },
    private val mockedGetPublishedKeysAndProfiles: suspend () -> PublishedKeysAndProfiles = {
        TODO("not mocked")
    },
    private val mockedGetDeadDrops: suspend (Int) -> PublishedJournalistToUserDeadDropsList = {
        TODO("not mocked")
    },
    private val mockedPostMessage: suspend (UserMessage) -> Unit = {
        TODO("not mocked")
    },
) : ICoverDropApiClient {

    override suspend fun getPublishedStatusEvent() =
        mockedGetStatusEvent()

    override suspend fun getPublishedKeys() =
        mockedGetPublishedKeysAndProfiles()

    override suspend fun getDeadDrops(idsGreaterThan: Int): PublishedJournalistToUserDeadDropsList =
        mockedGetDeadDrops(idsGreaterThan)

    override suspend fun postMessage(message: UserMessage) =
        mockedPostMessage(message)

    companion object {
        fun fromApiCallProvider(
            apiCallProvider: TestApiCallProvider,
            apiConfiguration: ApiConfiguration,
        ): CoverDropApiClientMock {
            return CoverDropApiClientMock(
                mockedGetStatusEvent = {
                    PublishedStatusEvent(
                        status = "AVAILABLE",
                        isAvailable = true,
                        description = "",
                        timestamp = Instant.now(),
                    )
                },
                mockedGetDeadDrops = {
                    GsonApiJsonAdapter().parsePublishedDeadDrops(
                        apiCallProvider.getJsonApi(
                            apiConfiguration = apiConfiguration,
                            path = "/v1/user/dead-drops",
                            queryParameters = listOf(Pair("ids_greater_than", "0")),
                        )
                    )
                },
                mockedGetPublishedKeysAndProfiles = {
                    GsonApiJsonAdapter().parsePublishedPublicKeys(
                        apiCallProvider.getJsonApi(
                            apiConfiguration = apiConfiguration,
                            path = "/v1/public-keys",
                        )
                    )
                },
            )
        }
    }
}
