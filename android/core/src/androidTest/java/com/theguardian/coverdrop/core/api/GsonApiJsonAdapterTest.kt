package com.theguardian.coverdrop.core.api

import androidx.test.platform.app.InstrumentationRegistry
import com.google.common.truth.Truth.assertThat
import com.theguardian.coverdrop.core.api.models.PublishedJournalistProfile
import com.theguardian.coverdrop.core.api.models.SystemStatus
import com.theguardian.coverdrop.core.models.JournalistInfo
import com.theguardian.coverdrop.core.models.JournalistVisibility
import com.theguardian.coverdrop.core.toJournalistInfo
import com.theguardian.coverdrop.testutils.InstantSubject
import com.theguardian.coverdrop.testutils.IntegrationTestVectors
import com.theguardian.coverdrop.testutils.TestScenario
import org.junit.Test
import java.time.Duration
import java.time.ZonedDateTime


class GsonApiJsonAdapterTest {

    private val context = InstrumentationRegistry.getInstrumentation().context
    private val instance = GsonApiJsonAdapter()

    @Test(expected = NullPointerException::class)
    fun parseUnverifiedPublicKeys_whenEmpty_thenThrows() {
        instance.parsePublishedPublicKeys("")
    }

    @Test
    fun parseStatusEvent_whenGivenTestVector1_thenReturnsExpected() {
        val testVectors = IntegrationTestVectors(context, TestScenario.SetSystemStatus)
        val json = testVectors.readJson("system_status", "001_initial_status.json")
        val parsed = instance.parsePublishedStatusEvent(json)

        assertThat(parsed.getStatus()).isEqualTo(SystemStatus.NO_INFORMATION)
        assertThat(parsed.isAvailable).isFalse()
        assertThat(parsed.description).isEqualTo("No information available")
        assertThat(parsed.timestamp).isNotNull()
    }

    @Test
    fun parseStatusEvent_whenGivenTestVector2_thenReturnsExpected() {
        val testVectors = IntegrationTestVectors(context, TestScenario.SetSystemStatus)
        val json = testVectors.readJson("system_status", "002_status_available.json")
        val parsed = instance.parsePublishedStatusEvent(json)

        assertThat(parsed.getStatus()).isEqualTo(SystemStatus.AVAILABLE)
        assertThat(parsed.isAvailable).isTrue()
        assertThat(parsed.description).isEqualTo("All good!")
        assertThat(parsed.timestamp).isNotNull()
    }

    @Test
    fun parseStatusEvent_whenGivenTestVector3_thenReturnsExpected() {
        val testVectors = IntegrationTestVectors(context, TestScenario.SetSystemStatus)
        val json = testVectors.readJson("system_status", "003_status_unavailable.json")
        val parsed = instance.parsePublishedStatusEvent(json)

        assertThat(parsed.getStatus()).isEqualTo(SystemStatus.UNAVAILABLE)
        assertThat(parsed.isAvailable).isFalse()
        assertThat(parsed.description).isEqualTo("CoverDrop is currently unavailable. We are working on a fix.")
        assertThat(parsed.timestamp).isNotNull()
    }

    @Test
    fun parseJournalistProfile_whenGivenHiddenStatus_thenCorrectlyInterpreted() {
        val json = "{" +
                "\"id\":\"rosalind_franklin\"," +
                "\"display_name\":\"Rosalind Franklin\"," +
                "\"sort_name\":\"franklin rosalind\"," +
                "\"description\":\"Chemistry correspondent\"," +
                "\"is_desk\":false," +
                "\"tag\":\"d6e0cc6c\"," +
                "\"status\":\"HIDDEN_FROM_UI\"}"

        val actual = createGsonInstance()
            .fromJson(json, PublishedJournalistProfile::class.java)
            .toJournalistInfo()
        val expected = JournalistInfo(
            id = "rosalind_franklin",
            displayName = "Rosalind Franklin",
            sortName = "franklin rosalind",
            description = "Chemistry correspondent",
            isTeam = false,
            tag = "d6e0cc6c",
            visibility = JournalistVisibility.HIDDEN
        )

        assertThat(actual).isEqualTo(expected)
    }

    @Test
    fun parseUnverifiedPublicKeys_whenTestVectorSampleResponse_thenMatches() {
        val testVectors = IntegrationTestVectors(context, TestScenario.Minimal)
        val json = testVectors.readJson("published_keys")
        val parsed = instance.parsePublishedPublicKeys(json)

        val journalistId = "static_test_journalist"
        val coverNodeId = "covernode_001"

        // check general key hierarchy
        val keyHierarchy = parsed.keys.single()
        assertThat(keyHierarchy.orgPk).isNotNull()

        // check coverNode key hierarchy
        val coverNodeKeyHierarchy = keyHierarchy.coverNodesKeyHierarchy.single()
        val testCoverNode = coverNodeKeyHierarchy.coverNodes[coverNodeId]!!.single()
        assertThat(coverNodeKeyHierarchy.provisioningPk).isNotNull()
        assertThat(testCoverNode.idPk).isNotNull()
        assertThat(testCoverNode.msgPks).hasSize(1)

        // check journalists key hierarchy
        val journalistsKeyHierarchy = keyHierarchy.journalistsKeyHierarchy.single()
        assertThat(journalistsKeyHierarchy.provisioningPk).isNotNull()
        assertThat(journalistsKeyHierarchy.journalists).hasSize(1)
        val testJournalistKeys = journalistsKeyHierarchy.journalists[journalistId]!!.single()
        assertThat(testJournalistKeys.idPk).isNotNull()
        assertThat(testJournalistKeys.msgPks).hasSize(1)

        // check journalist profiles
        val journalistProfiles = checkNotNull(parsed.journalistProfiles)
        val testJournalist = journalistProfiles.single()

        assertThat(testJournalist.id).isEqualTo(journalistId)
        assertThat(testJournalist.displayName).isEqualTo("Static Test Journalist")
        assertThat(testJournalist.sortName).isEqualTo("journalist static test")
        assertThat(testJournalist.description).isEqualTo("static test journalist")
        assertThat(testJournalist.isDesk).isEqualTo(false)
        assertThat(testJournalist.tag).isEqualTo("6a139e67")

        // check one leaf key to verify our parsing of the key, certificate, and timestamp
        val key = testJournalistKeys.msgPks.single()

        // this has the draw back the we'll need to manually update the following code lines whenever
        // the test vector changes. but it's the easiest way to verify the parsing of the key
        assertThat(key.key).isEqualTo("435858525a712c99ea5c0172197ea7313f491d093b09260ea503c54837f51637")
        assertThat(key.certificate).isEqualTo("ccb94722dc4bc9a0b6f0ff5cc46b38c148e2ea76584120c3cabe820a86856a55d0d04fb92ea27b69320a2ddf49d955da888e325732e2fcc960b265fc4d5e3003")
        assertThat(key.signature).isEqualTo(key.certificate)

        // this should be tracking `keys_generated_at.txt` for the test vector updates
        val expectedExpiryDate = ZonedDateTime.parse("2026-09-21T15:03:31Z").toInstant()
        InstantSubject.assertThat(expectedExpiryDate).isCloseTo(
            expected = key.notValidAfter,
            tolerance = Duration.ofSeconds(100)
        )
    }

}
