package com.theguardian.coverdrop.core.crypto

import androidx.test.platform.app.InstrumentationRegistry
import com.google.common.truth.Truth.assertThat
import com.theguardian.coverdrop.core.api.GsonApiJsonAdapter
import com.theguardian.coverdrop.core.api.models.PublishedJournalistToUserDeadDropsList
import com.theguardian.coverdrop.core.api.models.PublishedKeysAndProfiles
import com.theguardian.coverdrop.core.api.models.VerifiedCoverNodeKeyHierarchy
import com.theguardian.coverdrop.core.api.models.VerifiedKeyFamily
import com.theguardian.coverdrop.core.api.models.VerifiedSignedSigningKey
import com.theguardian.coverdrop.core.createLibSodium
import com.theguardian.coverdrop.core.generated.JOURNALIST_TO_USER_ENCRYPTED_MESSAGE_LEN
import com.theguardian.coverdrop.core.utils.hexDecode
import com.theguardian.coverdrop.core.utils.nextByteArray
import com.theguardian.coverdrop.testutils.IntegrationTestVectors
import com.theguardian.coverdrop.testutils.TestScenario
import org.junit.Test
import java.security.SecureRandom
import java.time.Instant
import kotlin.experimental.xor


class DeadDropParserTest {

    private val context = InstrumentationRegistry.getInstrumentation().context
    private val libSodium = createLibSodium()

    //
    // parseDeadDropData
    //

    @Test
    fun testParseDeadDropData_whenEmpty_thenEmptyResult() {
        val instance = DeadDropParser(libSodium, VerificationFailureBehaviour.THROW)

        val actual = instance.parseDeadDropData(byteArrayOf())
        assertThat(actual).isEmpty()
    }

    @Test(expected = IllegalArgumentException::class)
    fun testParseDeadDropData_whenNonDivisibleByChunkSize_thenThrows() {
        val instance = DeadDropParser(libSodium, VerificationFailureBehaviour.THROW)

        val random = SecureRandom()
        instance.parseDeadDropData(random.nextByteArray(2 * JOURNALIST_TO_USER_ENCRYPTED_MESSAGE_LEN - 1))
    }

    @Test
    fun testParseDeadDropData_whenCorrectlySized_thenResultsCombinedMatchInput() {
        val instance = DeadDropParser(libSodium, VerificationFailureBehaviour.THROW)

        val random = SecureRandom()
        val bytes = random.nextByteArray(4 * JOURNALIST_TO_USER_ENCRYPTED_MESSAGE_LEN)

        val actual = instance.parseDeadDropData(bytes)
        assertThat(actual).hasSize(4)

        val combined = actual.map { it.bytes }.reduce { a, b -> a + b }
        assertThat(combined).isEqualTo(bytes)
    }

    //
    // verifyAndParseDeadDropsListOrThrow
    //

    @Test
    fun testVerifyAndParseDeadDropsList_whenGivenCorrectTestVector_thenVerifies() {
        val instance = DeadDropParser(libSodium, VerificationFailureBehaviour.THROW)

        val integrationTestVectors = IntegrationTestVectors(context, TestScenario.Minimal)
        val publishedDeadDrops =
            GsonApiJsonAdapter().parsePublishedDeadDrops(integrationTestVectors.readJson("user_dead_drops"))
        val publishedKeys =
            GsonApiJsonAdapter().parsePublishedPublicKeys(integrationTestVectors.readJson("published_keys"))

        // we circumvent the verification here to not duplicate [KeyVerifierTest]
        val keyHierarchy = internalToCoverNodeKeyHierarchyForTest(publishedKeys)

        val verifiedDeadDrops = instance.verifyAndParseDeadDropsList(
            candidate = publishedDeadDrops,
            coverNodeKeyHierarchies = listOf(keyHierarchy),
        )

        assertThat(verifiedDeadDrops).hasSize(3)

        for (deadDrop in verifiedDeadDrops) {
            assertThat(deadDrop.id).isAnyOf(1, 2, 3)
            assertThat(deadDrop.createdAt).isAtLeast(Instant.parse("2000-01-01T00:00:00.000Z"))
            assertThat(deadDrop.messages).isNotEmpty()
        }
    }

    @Test(expected = DeadDropVerificationException::class)
    fun testVerifyAndParseDeadDropsList_whenGivenWrongCoverNodeKey_andThrowBehaviour_thenThrows() {
        val instance = DeadDropParser(libSodium, VerificationFailureBehaviour.THROW)

        val integrationTestVectors = IntegrationTestVectors(context, TestScenario.Minimal)
        val publishedDeadDrops =
            GsonApiJsonAdapter().parsePublishedDeadDrops(integrationTestVectors.readJson("user_dead_drops"))
        val publishedKeys =
            GsonApiJsonAdapter().parsePublishedPublicKeys(integrationTestVectors.readJson("published_keys"))

        // we circumvent the verification here to not duplicate [KeyVerifierTest]
        val keyHierarchy = internalToCoverNodeKeyHierarchyForTest(
            publishedKeys,
            flipBitInSigningKey = true
        )

        instance.verifyAndParseDeadDropsList(
            candidate = publishedDeadDrops,
            coverNodeKeyHierarchies = listOf(keyHierarchy),
        )
    }

    @Test
    fun testVerifyAndParseDeadDropsList_whenOneSignatureIsBad_andIgnoreBehaviour_thenAllOthersOk() {
        val instance = DeadDropParser(libSodium, VerificationFailureBehaviour.DROP)

        val integrationTestVectors = IntegrationTestVectors(context, TestScenario.Minimal)
        val publishedDeadDrops =
            GsonApiJsonAdapter().parsePublishedDeadDrops(integrationTestVectors.readJson("user_dead_drops"))
        val publishedKeys =
            GsonApiJsonAdapter().parsePublishedPublicKeys(integrationTestVectors.readJson("published_keys"))

        // we circumvent the verification here to not duplicate [KeyVerifierTest]
        val keyHierarchy = internalToCoverNodeKeyHierarchyForTest(publishedKeys)

        // we alter the signature of the second dead-drop
        val alteredPublishedDeadDrops = publishedDeadDrops.deadDrops.map { deadDrop ->
            if (deadDrop.id == 2) {
                deadDrop.copy(signature = deadDrop.signature!!.reversed())
            } else {
                deadDrop
            }
        }
        val alteredPublishedDeadDropsList = PublishedJournalistToUserDeadDropsList(
            deadDrops = alteredPublishedDeadDrops
        )

        // parse and verify
        val verifiedDeadDrops = instance.verifyAndParseDeadDropsList(
            candidate = alteredPublishedDeadDropsList,
            coverNodeKeyHierarchies = listOf(keyHierarchy),
        )

        assertThat(verifiedDeadDrops).hasSize(3 - 1)

        for (deadDrop in verifiedDeadDrops) {
            assertThat(deadDrop.id).isAnyOf(1, 3)
            assertThat(deadDrop.createdAt).isAtLeast(Instant.parse("2000-01-01T00:00:00.000Z"))
            assertThat(deadDrop.messages).isNotEmpty()
        }
    }

    //
    // Helper
    //

    private fun internalToCoverNodeKeyHierarchyForTest(
        publishedKeys: PublishedKeysAndProfiles,
        flipBitInSigningKey: Boolean = false,
    ): VerifiedCoverNodeKeyHierarchy {
        val publishedKeyHierarchy = publishedKeys.keys.single()
        val publishedCoverNodeKeyHierarchy = publishedKeyHierarchy.coverNodesKeyHierarchy.single()
        val publishedCoverNodeProvisioningKey =
            PublicSigningKey(publishedCoverNodeKeyHierarchy.provisioningPk.key.hexDecode())
        val publishedCoverNode = publishedCoverNodeKeyHierarchy.coverNodes.values.single()
        val publishedCoverNodeKeyFamily = publishedCoverNode.single()

        var rawSigningKey = PublicSigningKey(publishedCoverNodeKeyFamily.idPk.key.hexDecode())
        if (flipBitInSigningKey) {
            // flip a bit in the signing key to simulate a wrong key
            val bytes = rawSigningKey.bytes
            bytes[0] = bytes[0] xor 0x01
            rawSigningKey = PublicSigningKey(bytes)
        }

        val signedSigningKeys = listOf(VerifiedSignedSigningKey(rawSigningKey))

        return VerifiedCoverNodeKeyHierarchy(
            provisioningPk = VerifiedSignedSigningKey(publishedCoverNodeProvisioningKey), // will not be checked here
            coverNodes = mapOf(
                "covernode_001" to signedSigningKeys.map { VerifiedKeyFamily(it, emptyList()) },
            )
        )
    }
}
