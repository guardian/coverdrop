package com.theguardian.coverdrop.testutils

import android.content.Context
import com.theguardian.coverdrop.core.CoverDropConfiguration
import com.theguardian.coverdrop.core.api.ApiConfiguration
import com.theguardian.coverdrop.core.utils.IClock


fun createCoverDropConfigurationForTest(
    context: Context,
    scenario: TestScenario = TestScenario.Minimal,
    apiBaseUrl: String = "https://example.org",
    messagingBaseUrl: String = "https://example.org",
    clockOverride: IClock? = null,
): CoverDropConfiguration {
    val testVectors = IntegrationTestVectors(context, scenario)
    return CoverDropConfiguration(
        apiConfiguration = ApiConfiguration(
            apiBaseUrl = apiBaseUrl,
            messagingBaseUrl = messagingBaseUrl,
        ),
        clock = clockOverride ?: TestClock(
            nowOverride = testVectors.getNow(),
        ),
        createApiCallProvider = {
            TestApiCallProvider(testVectors)
        },
        trustedOrgPks = testVectors.getKeys().getTrustedOrganisationKeys(),
    )
}

fun createMinimalCoverDropTestConfigurationWithClock(clock: IClock): CoverDropConfiguration {
    return CoverDropConfiguration(
        apiConfiguration = ApiConfiguration(apiBaseUrl = "", messagingBaseUrl = ""),
        clock = clock,
        createApiCallProvider = { TODO("not mocked") },
        trustedOrgPks = emptyList(),
    )
}
