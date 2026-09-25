@file:Suppress("PrivatePropertyName")

package com.theguardian.coverdrop.core.crypto

import androidx.test.platform.app.InstrumentationRegistry
import com.google.common.truth.Truth.assertThat
import com.theguardian.coverdrop.core.api.GsonApiJsonAdapter
import com.theguardian.coverdrop.core.api.models.PublishedKeyFamily
import com.theguardian.coverdrop.core.api.models.PublishedSignedEncryptionKey
import com.theguardian.coverdrop.core.api.models.PublishedSignedSigningKey
import com.theguardian.coverdrop.core.api.models.VerifiedSignedSigningKey
import com.theguardian.coverdrop.core.createLibSodium
import com.theguardian.coverdrop.core.utils.hexDecode
import com.theguardian.coverdrop.core.utils.hexEncode
import com.theguardian.coverdrop.testutils.IntegrationTestVectors
import com.theguardian.coverdrop.testutils.TestScenario
import org.junit.Test
import java.time.Duration
import java.time.Instant

private const val TEST_ORG_PK = "f9162ddd3609f1985b9d00c1701c2dfa046c819eefc81d5b3a8b6799c27827ee"
private const val TEST_ORG_PK_CERTIFICATE =
    "a05beac4862a73bc56243c91686bad92bf209131d34d0225f1c7832c96931f3cdeed011203ffe95a9fea74428735c22f2f3a8092ca65f1521192b38be8060d0c"
private val TEST_ORG_NOT_VALID_AFTER = Instant.parse("2024-09-02T17:16:49.896447Z")

private val TEST_TRUSTED_ORG_PKS = listOf(
    PublicSigningKey("0000000000000000000000000000000000000000000000000000000000000001".hexDecode()),
    PublicSigningKey(TEST_ORG_PK.hexDecode()),
)

private const val TEST_SIGNING_KEY_PARENT =
    "079c046516e934e63e0ad92297ec8bab0408d195b697c31c8cb7af4e8eda805e"
private const val TEST_SIGNING_KEY =
    "51f026db091814be15ccaf5ba0ae6c032245b4f7ff43f978949115b9ca824a47"
private const val TEST_SIGNING_CERTIFICATE =
    "46c6ea31efb7faa8e2292ee2ff16ee8d12508156ef8af50b4aa46274060854aa3698633deca319fbc101fb1f5882534225f2726a79486fe7bbb5b020d3b02e05"
private val TEST_SIGNING_NOT_VALID_AFTER = Instant.parse("2024-02-07T19:35:43.470490Z")

private const val TEST_ENC_KEY_PARENT =
    "ae3b5686f57a7679d2f199aaec320ab5905532b4144edaddb5c37f8b0c59f1e3"
private const val TEST_ENC_KEY =
    "3a191e4368bb592634849d0c2ec1dafd252851426f4686461e2c36ab80c6d258"
private const val TEST_ENC_CERTIFICATE =
    "aaf2d313a88a5d8168dea7ea2877829a5637a2343ef972fdf5492c959420748442d5f20fef0f3de8d74cf6190b3e7f781106c1a661a1ed64662652cc17b62b04"
private val TEST_ENC_NOT_VALID_AFTER = Instant.parse("2023-08-30T19:35:46.004318Z")


class KeyVerifierTest {
    private val context = InstrumentationRegistry.getInstrumentation().context
    private val libSodium = createLibSodium()
    private val instance = KeyVerifier(libSodium)

    //
    // verifyPublishedKeysOrThrow
    //

    @Test
    fun testVerifyPublishedKeys_whenGivenCorrectTestVector_thenVerifies() {
        val testVectors = IntegrationTestVectors(context, TestScenario.Minimal)
        val publishedKeys = GsonApiJsonAdapter().parsePublishedPublicKeys(testVectors.readJson("published_keys"))

        // all keys in the current vectors carry the new signature field
        val hierarchy = publishedKeys.keys.single()
        assertThat(hierarchy.orgPk.signature).isNotNull()
        val coverNodesHierarchy = hierarchy.coverNodesKeyHierarchy.single()
        assertThat(coverNodesHierarchy.provisioningPk.signature).isNotNull()
        assertThat(coverNodesHierarchy.coverNodes.getValue("covernode_001").single().idPk.signature).isNotNull()
        val journalistsHierarchy = hierarchy.journalistsKeyHierarchy.single()
        assertThat(journalistsHierarchy.provisioningPk.signature).isNotNull()
        assertThat(journalistsHierarchy.journalists.getValue("static_test_journalist").single().idPk.signature).isNotNull()

        verifyAndAssertMinimalScenario(testVectors)
    }

    @Test
    fun testVerifyPublishedKeys_whenGivenOldTestVectorWithoutSignatures_thenVerifies() {
        val testVectors = IntegrationTestVectors(context, TestScenario.MinimalOldWithoutIdKeySignature)
        val publishedKeys = GsonApiJsonAdapter().parsePublishedPublicKeys(testVectors.readJson("published_keys"))

        val journalistIdPk = publishedKeys.keys.single().journalistsKeyHierarchy.single()
            .journalists.getValue("static_test_journalist").single().idPk
        assertThat(journalistIdPk.signature).isNull()

        verifyAndAssertMinimalScenario(testVectors)
    }

    @Test
    fun testVerifyPublishedKeys_whenJournalistKeysListedUnderOtherIdentity_thenOnlyOriginalVerifies() {
        val testVectors = IntegrationTestVectors(context, TestScenario.Minimal)
        val publishedKeys = GsonApiJsonAdapter().parsePublishedPublicKeys(testVectors.readJson("published_keys"))

        val journalists = publishedKeys.keys.single().journalistsKeyHierarchy.single().journalists
        val families = journalists.getValue("static_test_journalist")
        assertThat(families.single().idPk.signature).isNotNull()
        journalists["other_journalist"] = families

        val verifiedKeys = instance.verifyPublishedKeysAndProfiles(
            publishedKeysAndProfiles = publishedKeys,
            trustedOrgPks = testVectors.getKeys().getTrustedOrganisationKeys(),
            now = testVectors.getNow()
        )

        val verifiedJournalists = verifiedKeys.keys.single().journalistsHierarchies.single().journalists
        assertThat(verifiedJournalists.getValue("static_test_journalist")).hasSize(1)
        assertThat(verifiedJournalists.getValue("other_journalist")).isEmpty()
    }

    private fun verifyAndAssertMinimalScenario(testVectors: IntegrationTestVectors) {
        val json = testVectors.readJson("published_keys")
        val publishedKeys = GsonApiJsonAdapter().parsePublishedPublicKeys(json)

        val verifiedKeys = instance.verifyPublishedKeysAndProfiles(
            publishedKeysAndProfiles = publishedKeys,
            trustedOrgPks = testVectors.getKeys().getTrustedOrganisationKeys(),
            now = testVectors.getNow()
        )

        val keyHierarchy = verifiedKeys.keys.single()
        assertThat(keyHierarchy.orgPk.pk.bytes).isNotEmpty()

        val coverNodeKeyHierarchy = keyHierarchy.coverNodeHierarchies.single()
        assertThat(coverNodeKeyHierarchy.provisioningPk.pk.bytes).isNotEmpty()

        val coverNode = coverNodeKeyHierarchy.coverNodes.values.single()
        assertThat(coverNode).hasSize(1)

        val journalistsKeyHierarchy = keyHierarchy.journalistsHierarchies.single()
        assertThat(journalistsKeyHierarchy.provisioningPk.pk.bytes).isNotEmpty()

        val journalists = journalistsKeyHierarchy.journalists
        assertThat(journalists).hasSize(1)

        for (journalist in journalists) {
            val journalistKeys = journalist.value.single()
            assertThat(journalistKeys.idPk.pk.bytes).isNotEmpty()
            assertThat(journalistKeys.msgPks.size).isAtLeast(1)
            assertThat(journalistKeys.msgPks[0].pk.bytes).isNotEmpty()
        }
    }

    //
    // verifyTrustedRootKeyOrThrow
    //

    @Test(expected = IllegalArgumentException::class)
    fun testVerifyTrustedRootKey_whenInvalidKey_thenThrows() {
        val candidate = PublishedSignedSigningKey(
            key = "",
            certificate = "",
            notValidAfter = TEST_ORG_NOT_VALID_AFTER,
        )
        instance.verifyTrustedRootKeyOrThrow(
            orgPk = candidate,
            trustedOrgPks = TEST_TRUSTED_ORG_PKS,
            now = TEST_ORG_NOT_VALID_AFTER,
        )
    }

    @Test(expected = KeyVerificationException::class)
    fun testVerifyTrustedRootKey_whenInvalidCertificate_thenThrows() {
        val candidate = PublishedSignedSigningKey(
            key = TEST_ORG_PK,
            certificate = TEST_ORG_PK_CERTIFICATE.reversed(),
            notValidAfter = TEST_ORG_NOT_VALID_AFTER,
        )
        instance.verifyTrustedRootKeyOrThrow(
            orgPk = candidate,
            trustedOrgPks = TEST_TRUSTED_ORG_PKS,
            now = TEST_ORG_NOT_VALID_AFTER,
        )
    }

    @Test(expected = KeyMissingTrustAnchorException::class)
    fun testVerifyTrustedRootKey_whenEmptyTrustList_thenThrows() {
        val candidate = PublishedSignedSigningKey(
            key = TEST_ORG_PK,
            certificate = TEST_ORG_PK_CERTIFICATE,
            notValidAfter = TEST_ORG_NOT_VALID_AFTER,
        )
        instance.verifyTrustedRootKeyOrThrow(
            orgPk = candidate,
            trustedOrgPks = emptyList(),
            now = TEST_ORG_NOT_VALID_AFTER,
        )
    }

    @Test(expected = KeyMissingTrustAnchorException::class)
    fun testVerifyTrustedRootKey_whenKeyNotInTrustList_thenThrows() {
        val candidate = PublishedSignedSigningKey(
            key = TEST_ORG_PK,
            certificate = TEST_ORG_PK_CERTIFICATE,
            notValidAfter = TEST_ORG_NOT_VALID_AFTER,
        )
        instance.verifyTrustedRootKeyOrThrow(
            orgPk = candidate,
            trustedOrgPks = TEST_TRUSTED_ORG_PKS.subList(0, 1),
            now = TEST_ORG_NOT_VALID_AFTER,
        )
    }

    @Test
    fun testVerifyTrustedRootKey_whenKeyInTrustList_thenVerifies() {
        val candidate = PublishedSignedSigningKey(
            key = TEST_ORG_PK,
            certificate = TEST_ORG_PK_CERTIFICATE,
            notValidAfter = TEST_ORG_NOT_VALID_AFTER,
        )
        val orgPk = instance.verifyTrustedRootKeyOrThrow(
            orgPk = candidate,
            trustedOrgPks = TEST_TRUSTED_ORG_PKS,
            now = TEST_ORG_NOT_VALID_AFTER,
        )
        assertThat(orgPk).isNotNull()
    }

    //
    // verifyEncryptionKeyWithExpiryOrThrow
    //

    @Test(expected = IllegalArgumentException::class)
    fun testVerifyEncryptionKeyWithExpiryOrThrow_whenInvalidKey_thenThrows() {
        val candidate = PublishedSignedEncryptionKey(
            key = "", certificate = "", notValidAfter = TEST_ENC_NOT_VALID_AFTER
        )
        val now = TEST_ENC_NOT_VALID_AFTER
        instance.verifyEncryptionKeyWithExpiryOrThrow(
            candidate = candidate,
            parent = PublicSigningKey(TEST_SIGNING_KEY_PARENT.hexDecode()),
            now = now
        )
    }

    @Test(expected = KeyVerificationException::class)
    fun testVerifyEncryptionKeyWithExpiryOrThrow_whenWrongParentKey_thenThrows() {
        val wrongParentKey = TEST_ENC_KEY_PARENT.reversed()
        val candidate = PublishedSignedEncryptionKey(
            key = TEST_ENC_KEY,
            certificate = TEST_ENC_CERTIFICATE,
            notValidAfter = TEST_ENC_NOT_VALID_AFTER
        )
        instance.verifyEncryptionKeyWithExpiryOrThrow(
            candidate = candidate,
            parent = PublicSigningKey(wrongParentKey.hexDecode()),
            now = TEST_ENC_NOT_VALID_AFTER
        )
    }

    @Test(expected = KeyVerificationException::class)
    fun testVerifyEncryptionKeyWithExpiryOrThrow_whenWrongCertificate_thenThrows() {
        val wrongCertificate = TEST_ENC_CERTIFICATE.reversed()
        val candidate = PublishedSignedEncryptionKey(
            key = TEST_ENC_KEY,
            certificate = wrongCertificate,
            notValidAfter = TEST_ENC_NOT_VALID_AFTER
        )
        instance.verifyEncryptionKeyWithExpiryOrThrow(
            candidate = candidate,
            parent = PublicSigningKey(TEST_ENC_KEY_PARENT.hexDecode()),
            now = TEST_ENC_NOT_VALID_AFTER
        )
    }

    @Test(expected = KeyVerificationException::class)
    fun testVerifyEncryptionKeyWithExpiryOrThrow_whenExpired_thenThrows() {
        val candidate = PublishedSignedEncryptionKey(
            key = TEST_ENC_KEY,
            certificate = TEST_ENC_CERTIFICATE,
            notValidAfter = TEST_ENC_NOT_VALID_AFTER
        )
        val now = TEST_ENC_NOT_VALID_AFTER.plus(Duration.ofSeconds(1))
        instance.verifyEncryptionKeyWithExpiryOrThrow(
            candidate = candidate,
            parent = PublicSigningKey(TEST_ENC_KEY_PARENT.hexDecode()),
            now = now
        )
    }

    @Test
    fun testVerifyEncryptionKeyWithExpiryOrThrow_whenCorrectCertificateAndNotExpired_thenVerifies() {
        val candidate = PublishedSignedEncryptionKey(
            key = TEST_ENC_KEY,
            certificate = TEST_ENC_CERTIFICATE,
            notValidAfter = TEST_ENC_NOT_VALID_AFTER
        )
        instance.verifyEncryptionKeyWithExpiryOrThrow(
            candidate = candidate,
            parent = PublicSigningKey(TEST_ENC_KEY_PARENT.hexDecode()),
            now = TEST_ENC_NOT_VALID_AFTER
        )
    }

    //
    // verifyEncryptionKeyWithExpiryOrNull
    //

    @Test(expected = IllegalArgumentException::class)
    fun testVerifyEncryptionKeyWithExpiryOrNull_whenInvalidKey_thenThrows() {
        val candidate = PublishedSignedEncryptionKey(
            key = "", certificate = "", notValidAfter = TEST_ENC_NOT_VALID_AFTER
        )
        val now = TEST_ENC_NOT_VALID_AFTER
        instance.verifyEncryptionKeyWithExpiryOrNull(
            candidate = candidate,
            parent = PublicSigningKey(TEST_SIGNING_KEY_PARENT.hexDecode()),
            now = now
        )
    }

    @Test
    fun testVerifyEncryptionKeyWithExpiryOrNull_whenWrongParentKey_thenNull() {
        val wrongParentKey = TEST_ENC_KEY_PARENT.reversed()
        val candidate = PublishedSignedEncryptionKey(
            key = TEST_ENC_KEY,
            certificate = TEST_ENC_CERTIFICATE,
            notValidAfter = TEST_ENC_NOT_VALID_AFTER
        )
        assertThat(
            instance.verifyEncryptionKeyWithExpiryOrNull(
                candidate = candidate,
                parent = PublicSigningKey(wrongParentKey.hexDecode()),
                now = TEST_ENC_NOT_VALID_AFTER
            )
        ).isNull()
    }

    @Test
    fun testVerifyEncryptionKeyWithExpiryOrNull_whenWrongCertificate_thenNull() {
        val wrongCertificate = TEST_ENC_CERTIFICATE.reversed()
        val candidate = PublishedSignedEncryptionKey(
            key = TEST_ENC_KEY,
            certificate = wrongCertificate,
            notValidAfter = TEST_ENC_NOT_VALID_AFTER
        )
        assertThat(
            instance.verifyEncryptionKeyWithExpiryOrNull(
                candidate = candidate,
                parent = PublicSigningKey(TEST_ENC_KEY_PARENT.hexDecode()),
                now = TEST_ENC_NOT_VALID_AFTER
            )
        ).isNull()
    }

    @Test
    fun testVerifyEncryptionKeyWithExpiryOrNull_whenExpired_thenNull() {
        val candidate = PublishedSignedEncryptionKey(
            key = TEST_ENC_KEY,
            certificate = TEST_ENC_CERTIFICATE,
            notValidAfter = TEST_ENC_NOT_VALID_AFTER
        )
        val now = TEST_ENC_NOT_VALID_AFTER.plus(Duration.ofSeconds(1))
        assertThat(
            instance.verifyEncryptionKeyWithExpiryOrNull(
                candidate = candidate,
                parent = PublicSigningKey(TEST_ENC_KEY_PARENT.hexDecode()),
                now = now
            )
        ).isNull()
    }

    @Test
    fun testVerifyEncryptionKeyWithExpiryOrNull_whenCorrectCertificateAndNotExpired_thenVerifies() {
        val candidate = PublishedSignedEncryptionKey(
            key = TEST_ENC_KEY,
            certificate = TEST_ENC_CERTIFICATE,
            notValidAfter = TEST_ENC_NOT_VALID_AFTER
        )
        instance.verifyEncryptionKeyWithExpiryOrNull(
            candidate = candidate,
            parent = PublicSigningKey(TEST_ENC_KEY_PARENT.hexDecode()),
            now = TEST_ENC_NOT_VALID_AFTER
        )
    }

    //
    // verifySigningKeyWithExpiryOrThrow
    //

    @Test(expected = IllegalArgumentException::class)
    fun testVerifySigningKeyWithExpiryOrThrow_whenInvalidKey_thenThrows() {
        val candidate = PublishedSignedSigningKey(
            key = "", certificate = "", notValidAfter = TEST_SIGNING_NOT_VALID_AFTER
        )
        val now = TEST_SIGNING_NOT_VALID_AFTER
        instance.verifySigningKeyWithExpiryOrThrow(
            candidate = candidate,
            parent = PublicSigningKey(TEST_SIGNING_KEY_PARENT.hexDecode()),
            now = now
        )
    }

    @Test(expected = KeyVerificationException::class)
    fun testVerifySigningKeyWithExpiryOrThrow_whenWrongParentKey_thenThrows() {
        val wrongParentKey = TEST_SIGNING_KEY_PARENT.reversed()
        val candidate = PublishedSignedSigningKey(
            key = TEST_SIGNING_KEY,
            certificate = TEST_SIGNING_CERTIFICATE,
            notValidAfter = TEST_SIGNING_NOT_VALID_AFTER
        )
        instance.verifySigningKeyWithExpiryOrThrow(
            candidate = candidate,
            parent = PublicSigningKey(wrongParentKey.hexDecode()),
            now = TEST_SIGNING_NOT_VALID_AFTER
        )
    }

    @Test(expected = KeyVerificationException::class)
    fun testVerifySigningKeyWithExpiryOrThrow_whenWrongCertificate_thenThrows() {
        val wrongCertificate = TEST_SIGNING_CERTIFICATE.reversed()
        val candidate = PublishedSignedSigningKey(
            key = TEST_SIGNING_KEY,
            certificate = wrongCertificate,
            notValidAfter = TEST_SIGNING_NOT_VALID_AFTER
        )
        instance.verifySigningKeyWithExpiryOrThrow(
            candidate = candidate,
            parent = PublicSigningKey(TEST_SIGNING_KEY_PARENT.hexDecode()),
            now = TEST_SIGNING_NOT_VALID_AFTER
        )
    }

    @Test(expected = KeyVerificationException::class)
    fun testVerifySigningKeyWithExpiryOrThrow_whenExpired_thenThrows() {
        val candidate = PublishedSignedSigningKey(
            key = TEST_SIGNING_KEY,
            certificate = TEST_SIGNING_CERTIFICATE,
            notValidAfter = TEST_SIGNING_NOT_VALID_AFTER
        )
        val now = TEST_SIGNING_NOT_VALID_AFTER.plus(Duration.ofSeconds(1))
        instance.verifySigningKeyWithExpiryOrThrow(
            candidate = candidate,
            parent = PublicSigningKey(TEST_SIGNING_KEY_PARENT.hexDecode()),
            now = now
        )
    }

    @Test
    fun testVerifySigningKeyWithExpiryOrThrow_whsIGNINGorrectCertificateAndNotExpired_thenVerifies() {
        val candidate = PublishedSignedSigningKey(
            key = TEST_SIGNING_KEY,
            certificate = TEST_SIGNING_CERTIFICATE,
            notValidAfter = TEST_SIGNING_NOT_VALID_AFTER
        )
        instance.verifySigningKeyWithExpiryOrThrow(
            candidate = candidate,
            parent = PublicSigningKey(TEST_SIGNING_KEY_PARENT.hexDecode()),
            now = TEST_SIGNING_NOT_VALID_AFTER
        )
    }

    //
    // verifySigningKeyWithExpiryOrNull
    //

    @Test(expected = IllegalArgumentException::class)
    fun testVerifySigningKeyWithExpiryOrNull_whenInvalidKey_thenThrows() {
        val candidate = PublishedSignedSigningKey(
            key = "", certificate = "", notValidAfter = TEST_SIGNING_NOT_VALID_AFTER
        )
        val now = TEST_SIGNING_NOT_VALID_AFTER
        instance.verifySigningKeyWithExpiryOrNull(
            candidate = candidate,
            parent = PublicSigningKey(TEST_SIGNING_KEY_PARENT.hexDecode()),
            now = now
        )
    }

    @Test
    fun testVerifySigningKeyWithExpiryOrNull_whenWrongParentKey_thenNull() {
        val wrongParentKey = TEST_SIGNING_KEY_PARENT.reversed()
        val candidate = PublishedSignedSigningKey(
            key = TEST_SIGNING_KEY,
            certificate = TEST_SIGNING_CERTIFICATE,
            notValidAfter = TEST_SIGNING_NOT_VALID_AFTER
        )
        assertThat(
            instance.verifySigningKeyWithExpiryOrNull(
                candidate = candidate,
                parent = PublicSigningKey(wrongParentKey.hexDecode()),
                now = TEST_SIGNING_NOT_VALID_AFTER
            )
        ).isNull()
    }

    @Test
    fun testVerifySigningKeyWithExpiryOrNull_whenWrongCertificate_thenNull() {
        val wrongCertificate = TEST_SIGNING_CERTIFICATE.reversed()
        val candidate = PublishedSignedSigningKey(
            key = TEST_SIGNING_KEY,
            certificate = wrongCertificate,
            notValidAfter = TEST_SIGNING_NOT_VALID_AFTER
        )
        assertThat(
            instance.verifySigningKeyWithExpiryOrNull(
                candidate = candidate,
                parent = PublicSigningKey(TEST_SIGNING_KEY_PARENT.hexDecode()),
                now = TEST_SIGNING_NOT_VALID_AFTER
            )
        ).isNull()
    }

    @Test
    fun testVerifySigningKeyWithExpiryOrNull_whenExpired_thenNull() {
        val candidate = PublishedSignedSigningKey(
            key = TEST_SIGNING_KEY,
            certificate = TEST_SIGNING_CERTIFICATE,
            notValidAfter = TEST_SIGNING_NOT_VALID_AFTER
        )
        val now = TEST_SIGNING_NOT_VALID_AFTER.plus(Duration.ofSeconds(1))
        assertThat(
            instance.verifySigningKeyWithExpiryOrNull(
                candidate = candidate,
                parent = PublicSigningKey(TEST_SIGNING_KEY_PARENT.hexDecode()),
                now = now
            )
        ).isNull()
    }

    @Test
    fun testVerifySigningKeyWithExpiryOrNull_whenSignatureCorrectCertificateAndNotExpired_thenVerifies() {
        val candidate = PublishedSignedSigningKey(
            key = TEST_SIGNING_KEY,
            certificate = TEST_SIGNING_CERTIFICATE,
            notValidAfter = TEST_SIGNING_NOT_VALID_AFTER
        )
        instance.verifySigningKeyWithExpiryOrNull(
            candidate = candidate,
            parent = PublicSigningKey(TEST_SIGNING_KEY_PARENT.hexDecode()),
            now = TEST_SIGNING_NOT_VALID_AFTER
        )
    }

    //
    // signature field (plain signing and encryption keys)
    //

    @Test
    fun testVerifySigningKeyWithExpiryOrThrow_whenSignaturePresentAndValid_thenVerifies() {
        val candidate = PublishedSignedSigningKey(
            key = TEST_SIGNING_KEY,
            certificate = TEST_SIGNING_CERTIFICATE,
            notValidAfter = TEST_SIGNING_NOT_VALID_AFTER,
            signature = TEST_SIGNING_CERTIFICATE,
        )
        instance.verifySigningKeyWithExpiryOrThrow(
            candidate = candidate,
            parent = PublicSigningKey(TEST_SIGNING_KEY_PARENT.hexDecode()),
            now = TEST_SIGNING_NOT_VALID_AFTER
        )
    }

    @Test(expected = KeyVerificationException::class)
    fun testVerifySigningKeyWithExpiryOrThrow_whenSignaturePresentButInvalid_thenThrows() {
        val candidate = PublishedSignedSigningKey(
            key = TEST_SIGNING_KEY,
            certificate = TEST_SIGNING_CERTIFICATE,
            notValidAfter = TEST_SIGNING_NOT_VALID_AFTER,
            signature = TEST_SIGNING_CERTIFICATE.reversed(),
        )
        instance.verifySigningKeyWithExpiryOrThrow(
            candidate = candidate,
            parent = PublicSigningKey(TEST_SIGNING_KEY_PARENT.hexDecode()),
            now = TEST_SIGNING_NOT_VALID_AFTER
        )
    }

    @Test(expected = KeyVerificationException::class)
    fun testVerifySigningKeyWithExpiryOrThrow_whenSignatureMalformed_thenThrows() {
        val candidate = PublishedSignedSigningKey(
            key = TEST_SIGNING_KEY,
            certificate = TEST_SIGNING_CERTIFICATE,
            notValidAfter = TEST_SIGNING_NOT_VALID_AFTER,
            signature = "",
        )
        instance.verifySigningKeyWithExpiryOrThrow(
            candidate = candidate,
            parent = PublicSigningKey(TEST_SIGNING_KEY_PARENT.hexDecode()),
            now = TEST_SIGNING_NOT_VALID_AFTER
        )
    }

    @Test(expected = KeyVerificationException::class)
    fun testVerifySigningKeyWithExpiryOrThrow_whenSignatureValidButCertificateInvalid_thenThrows() {
        val candidate = PublishedSignedSigningKey(
            key = TEST_SIGNING_KEY,
            certificate = TEST_SIGNING_CERTIFICATE.reversed(),
            notValidAfter = TEST_SIGNING_NOT_VALID_AFTER,
            signature = TEST_SIGNING_CERTIFICATE,
        )
        instance.verifySigningKeyWithExpiryOrThrow(
            candidate = candidate,
            parent = PublicSigningKey(TEST_SIGNING_KEY_PARENT.hexDecode()),
            now = TEST_SIGNING_NOT_VALID_AFTER
        )
    }

    @Test
    fun testVerifySigningKeyWithExpiryOrNull_whenSignaturePresentButInvalid_thenNull() {
        val candidate = PublishedSignedSigningKey(
            key = TEST_SIGNING_KEY,
            certificate = TEST_SIGNING_CERTIFICATE,
            notValidAfter = TEST_SIGNING_NOT_VALID_AFTER,
            signature = TEST_SIGNING_CERTIFICATE.reversed(),
        )
        assertThat(
            instance.verifySigningKeyWithExpiryOrNull(
                candidate = candidate,
                parent = PublicSigningKey(TEST_SIGNING_KEY_PARENT.hexDecode()),
                now = TEST_SIGNING_NOT_VALID_AFTER
            )
        ).isNull()
    }

    @Test
    fun testVerifyEncryptionKeyWithExpiryOrThrow_whenSignaturePresentAndValid_thenVerifies() {
        val candidate = PublishedSignedEncryptionKey(
            key = TEST_ENC_KEY,
            certificate = TEST_ENC_CERTIFICATE,
            notValidAfter = TEST_ENC_NOT_VALID_AFTER,
            signature = TEST_ENC_CERTIFICATE,
        )
        instance.verifyEncryptionKeyWithExpiryOrThrow(
            candidate = candidate,
            parent = PublicSigningKey(TEST_ENC_KEY_PARENT.hexDecode()),
            now = TEST_ENC_NOT_VALID_AFTER
        )
    }

    @Test(expected = KeyVerificationException::class)
    fun testVerifyEncryptionKeyWithExpiryOrThrow_whenSignaturePresentButInvalid_thenThrows() {
        val candidate = PublishedSignedEncryptionKey(
            key = TEST_ENC_KEY,
            certificate = TEST_ENC_CERTIFICATE,
            notValidAfter = TEST_ENC_NOT_VALID_AFTER,
            signature = TEST_ENC_CERTIFICATE.reversed(),
        )
        instance.verifyEncryptionKeyWithExpiryOrThrow(
            candidate = candidate,
            parent = PublicSigningKey(TEST_ENC_KEY_PARENT.hexDecode()),
            now = TEST_ENC_NOT_VALID_AFTER
        )
    }

    @Test
    fun testVerifyEncryptionKeyWithExpiryOrNull_whenSignaturePresentButInvalid_thenNull() {
        val candidate = PublishedSignedEncryptionKey(
            key = TEST_ENC_KEY,
            certificate = TEST_ENC_CERTIFICATE,
            notValidAfter = TEST_ENC_NOT_VALID_AFTER,
            signature = TEST_ENC_CERTIFICATE.reversed(),
        )
        assertThat(
            instance.verifyEncryptionKeyWithExpiryOrNull(
                candidate = candidate,
                parent = PublicSigningKey(TEST_ENC_KEY_PARENT.hexDecode()),
                now = TEST_ENC_NOT_VALID_AFTER
            )
        ).isNull()
    }

    //
    // verifyIdSigningKeyWithExpiryOrThrow / OrNull
    //

    private val idKeyNow = Instant.parse("2025-01-01T00:00:00Z")
    private val idKeyNotValidAfter = idKeyNow.plus(Duration.ofDays(1))

    private fun newSignedSigningKey(
        parent: SigningKeyPair,
        notValidAfter: Instant = idKeyNotValidAfter,
        signatureIdentity: String? = null,
        signatureRole: IdKeyRole = IdKeyRole.JOURNALIST_ID,
        includeSignature: Boolean = true,
    ): PublishedSignedSigningKey {
        val child = SigningKeyPair.new(libSodium)
        val certificateData = SigningKeyCertificateData.from(child.publicSigningKey, notValidAfter)
        val signatureData: Signable = when (signatureIdentity) {
            null -> certificateData
            else -> IdKeyCertificateData.from(
                key = child.publicSigningKey,
                notValidAfter = notValidAfter,
                role = signatureRole,
                identity = signatureIdentity,
            )
        }
        return PublishedSignedSigningKey(
            key = child.publicSigningKey.bytes.hexEncode(),
            certificate = Signature.sign(libSodium, parent.secretSigningKey, certificateData).bytes.hexEncode(),
            notValidAfter = notValidAfter,
            signature = when (includeSignature) {
                true -> Signature.sign(libSodium, parent.secretSigningKey, signatureData).bytes.hexEncode()
                false -> null
            },
        )
    }

    @Test
    fun testVerifyIdSigningKeyWithExpiryOrThrow_whenSignatureForIdentity_thenVerifies() {
        val parent = SigningKeyPair.new(libSodium)
        val candidate = newSignedSigningKey(parent, signatureIdentity = "journalist_a")
        val verified = instance.verifyIdSigningKeyWithExpiryOrThrow(
            candidate = candidate,
            parent = parent.publicSigningKey,
            role = IdKeyRole.JOURNALIST_ID,
            identity = "journalist_a",
            now = idKeyNow
        )
        assertThat(verified.pk.bytes.hexEncode()).isEqualTo(candidate.key)
    }

    @Test(expected = KeyVerificationException::class)
    fun testVerifyIdSigningKeyWithExpiryOrThrow_whenSignatureForOtherIdentity_thenThrows() {
        val parent = SigningKeyPair.new(libSodium)
        val candidate = newSignedSigningKey(parent, signatureIdentity = "journalist_a")
        instance.verifyIdSigningKeyWithExpiryOrThrow(
            candidate = candidate,
            parent = parent.publicSigningKey,
            role = IdKeyRole.JOURNALIST_ID,
            identity = "journalist_b",
            now = idKeyNow
        )
    }

    @Test
    fun testVerifyIdSigningKeyWithExpiryOrNull_whenSignatureForOtherIdentity_thenNull() {
        val parent = SigningKeyPair.new(libSodium)
        val candidate = newSignedSigningKey(parent, signatureIdentity = "journalist_a")
        assertThat(
            instance.verifyIdSigningKeyWithExpiryOrNull(
                candidate = candidate,
                parent = parent.publicSigningKey,
                role = IdKeyRole.JOURNALIST_ID,
                identity = "journalist_b",
                now = idKeyNow
            )
        ).isNull()
    }

    @Test(expected = KeyVerificationException::class)
    fun testVerifyIdSigningKeyWithExpiryOrThrow_whenSignatureForOtherRole_thenThrows() {
        val parent = SigningKeyPair.new(libSodium)
        val candidate = newSignedSigningKey(
            parent,
            signatureIdentity = "journalist_a",
            signatureRole = IdKeyRole.COVERNODE_ID,
        )
        instance.verifyIdSigningKeyWithExpiryOrThrow(
            candidate = candidate,
            parent = parent.publicSigningKey,
            role = IdKeyRole.JOURNALIST_ID,
            identity = "journalist_a",
            now = idKeyNow
        )
    }

    @Test
    fun testVerifyIdSigningKeyWithExpiryOrNull_whenSignatureForOtherRole_thenNull() {
        val parent = SigningKeyPair.new(libSodium)
        val candidate = newSignedSigningKey(
            parent,
            signatureIdentity = "journalist_a",
            signatureRole = IdKeyRole.COVERNODE_ID,
        )
        assertThat(
            instance.verifyIdSigningKeyWithExpiryOrNull(
                candidate = candidate,
                parent = parent.publicSigningKey,
                role = IdKeyRole.JOURNALIST_ID,
                identity = "journalist_a",
                now = idKeyNow
            )
        ).isNull()
    }

    @Test
    fun testVerifyIdSigningKeyWithExpiryOrThrow_whenSignatureAbsent_thenVerifies() {
        val parent = SigningKeyPair.new(libSodium)
        val candidate = newSignedSigningKey(parent, includeSignature = false)
        assertThat(candidate.signature).isNull()
        instance.verifyIdSigningKeyWithExpiryOrThrow(
            candidate = candidate,
            parent = parent.publicSigningKey,
            role = IdKeyRole.JOURNALIST_ID,
            identity = "journalist_a",
            now = idKeyNow
        )
    }

    @Test(expected = KeyVerificationException::class)
    fun testVerifyIdSigningKeyWithExpiryOrThrow_whenSignatureOverPlainCertificateData_thenThrows() {
        val parent = SigningKeyPair.new(libSodium)
        val candidate = newSignedSigningKey(parent, signatureIdentity = null)
        instance.verifyIdSigningKeyWithExpiryOrThrow(
            candidate = candidate,
            parent = parent.publicSigningKey,
            role = IdKeyRole.JOURNALIST_ID,
            identity = "journalist_a",
            now = idKeyNow
        )
    }

    @Test(expected = KeyVerificationException::class)
    fun testVerifyIdSigningKeyWithExpiryOrThrow_whenSignatureInvalid_thenThrows() {
        val parent = SigningKeyPair.new(libSodium)
        val candidate = newSignedSigningKey(parent, signatureIdentity = "journalist_a")
        instance.verifyIdSigningKeyWithExpiryOrThrow(
            candidate = candidate.copy(signature = candidate.signature!!.reversed()),
            parent = parent.publicSigningKey,
            role = IdKeyRole.JOURNALIST_ID,
            identity = "journalist_a",
            now = idKeyNow
        )
    }

    @Test(expected = KeyVerificationException::class)
    fun testVerifyIdSigningKeyWithExpiryOrThrow_whenWrongParentKey_thenThrows() {
        val parent = SigningKeyPair.new(libSodium)
        val candidate = newSignedSigningKey(parent, signatureIdentity = "journalist_a")
        instance.verifyIdSigningKeyWithExpiryOrThrow(
            candidate = candidate,
            parent = SigningKeyPair.new(libSodium).publicSigningKey,
            role = IdKeyRole.JOURNALIST_ID,
            identity = "journalist_a",
            now = idKeyNow
        )
    }

    @Test(expected = KeyExpirationException::class)
    fun testVerifyIdSigningKeyWithExpiryOrThrow_whenExpired_thenThrowsExpiration() {
        val parent = SigningKeyPair.new(libSodium)
        val candidate = newSignedSigningKey(parent, signatureIdentity = "journalist_a")
        instance.verifyIdSigningKeyWithExpiryOrThrow(
            candidate = candidate,
            parent = parent.publicSigningKey,
            role = IdKeyRole.JOURNALIST_ID,
            identity = "journalist_b",
            now = idKeyNotValidAfter.plus(Duration.ofSeconds(1))
        )
    }

    @Test
    fun testVerifyKeyFamilies_whenIdKeySignedForOtherIdentity_thenFamilyDropped() {
        val parent = SigningKeyPair.new(libSodium)
        val provisioningKey = VerifiedSignedSigningKey(pk = parent.publicSigningKey)
        val families = listOf(
            PublishedKeyFamily(
                idPk = newSignedSigningKey(parent, signatureIdentity = "journalist_a"),
                msgPks = emptyList()
            )
        )

        assertThat(
            instance.verifyKeyFamilies(families, provisioningKey, IdKeyRole.JOURNALIST_ID, "journalist_a", idKeyNow)
        ).hasSize(1)
        assertThat(
            instance.verifyKeyFamilies(families, provisioningKey, IdKeyRole.JOURNALIST_ID, "journalist_b", idKeyNow)
        ).isEmpty()
        assertThat(
            instance.verifyKeyFamilies(families, provisioningKey, IdKeyRole.COVERNODE_ID, "journalist_a", idKeyNow)
        ).isEmpty()
    }
}
