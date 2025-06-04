package com.theguardian.coverdrop.core.api.models

import androidx.test.platform.app.InstrumentationRegistry
import com.google.common.truth.Truth.assertThat
import com.theguardian.coverdrop.core.api.GsonApiJsonAdapter
import com.theguardian.coverdrop.core.createLibSodium
import com.theguardian.coverdrop.core.crypto.KeyVerifier
import com.theguardian.coverdrop.testutils.IntegrationTestVectors
import com.theguardian.coverdrop.testutils.TestClock
import com.theguardian.coverdrop.testutils.TestScenario
import org.junit.Test
import java.time.Duration
import java.time.Instant

class VerifiedKeysTest {

    private val context = InstrumentationRegistry.getInstrumentation().context
    private val keyVerifier = KeyVerifier(createLibSodium())

    @Test
    fun mostRecentMessagingKeyForEachCoverNode_whenOneValidKey_thenItReturned() {
        val (verifiedKeys, now) = getVerifiedKeysAndInstant(
            testScenario = TestScenario.Messaging,
            filename = "003_journalist_replied_and_processed.json"
        )

        val actual = verifiedKeys.mostRecentMessagingKeyForEachCoverNode(TestClock(now))
        assertThat(actual.values).hasSize(1)
    }

    @Test
    fun mostRecentMessagingKeyForEachCoverNode_whenTwoValidKeys_thenLatestReturned() {
        val (verifiedKeys, now) = getVerifiedKeysAndInstant(
            testScenario = TestScenario.KeyRotations,
            filename = "003_covernode_msg_rotated_2.json"
        )

        val actual = verifiedKeys.mostRecentMessagingKeyForEachCoverNode(TestClock(now))
        assertThat(actual.values).hasSize(1)
        assertThat(actual.values.single().notValidAfter).isGreaterThan(Instant.parse("2023-10-27T00:00:00.0Z"))
    }

    @Test
    fun mostRecentMessagingKeyForEachCoverNode_whenPastOnlyValidKey_thenNothingReturned() {
        val (verifiedKeys, now) = getVerifiedKeysAndInstant(
            testScenario = TestScenario.Messaging,
            filename = "003_journalist_replied_and_processed.json"
        )
        val later = now + Duration.ofDays(28)

        val actual = verifiedKeys.mostRecentMessagingKeyForEachCoverNode(TestClock(later))
        assertThat(actual.values).hasSize(0)
    }

    @Test
    fun mostRecentMessagingKeyForJournalist_whenOneValidKeys_thenItReturned() {
        val (verifiedKeys, now) = getVerifiedKeysAndInstant(
            testScenario = TestScenario.Messaging,
            filename = "003_journalist_replied_and_processed.json"
        )

        val actual = verifiedKeys.mostRecentMessagingKeyForJournalist(
            journalistId = "static_test_journalist",
            clock = TestClock(now)
        )
        assertThat(actual.notValidAfter).isGreaterThan(now)
    }

    @Test
    fun mostRecentMessagingKeyForJournalist_whenThreeValidKeys_thenLatestReturned() {
        val (verifiedKeys, now) = getVerifiedKeysAndInstant(
            testScenario = TestScenario.KeyRotations,
            filename = "003_covernode_msg_rotated_2.json"
        )

        val actual = verifiedKeys.mostRecentMessagingKeyForJournalist(
            journalistId = "static_test_journalist",
            clock = TestClock(now)
        )
        assertThat(actual.notValidAfter).isGreaterThan(Instant.parse("2023-10-27T00:00:00.0Z"))
    }

    @Test(expected = IllegalStateException::class)
    fun mostRecentMessagingKeyForJournalist_whenPastOneValidKeys_thenThrows() {
        val (verifiedKeys, now) = getVerifiedKeysAndInstant(
            testScenario = TestScenario.Messaging,
            filename = "003_journalist_replied_and_processed.json"
        )
        val later = now + Duration.ofDays(28)

        verifiedKeys.mostRecentMessagingKeyForJournalist(
            journalistId = "static_test_journalist",
            clock = TestClock(later)
        )
    }

    private fun getVerifiedKeysAndInstant(
        testScenario: TestScenario,
        filename: String,
    ): Pair<VerifiedKeys, Instant> {
        val testVectors = IntegrationTestVectors(context, testScenario)
        val json = testVectors.readJson("published_keys", filename)
        val publishedKeys = GsonApiJsonAdapter().parsePublishedPublicKeys(json)

        val verifiedKeys = keyVerifier.verifyPublishedKeysAndProfiles(
            publishedKeysAndProfiles = publishedKeys,
            trustedOrgPks = testVectors.getKeys().getTrustedOrganisationKeys(),
            now = testVectors.getNow()
        )
        val clock = testVectors.getNow()

        return Pair(verifiedKeys, clock)
    }
}
