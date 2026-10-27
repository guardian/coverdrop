@testable import CoverDropCore
import Sodium
import XCTest

final class KeyVerificationTests: XCTestCase {
    func testSuccessfulVerification() throws {
        let orgPublicKey = try PublicKeysHelper.readLocalKeypairFile(path: "organization").publicKey
        let organizationSigningKey = try XCTUnwrap(OrganizationPublicKey(
            key: Sign.KeyPair.PublicKey(orgPublicKey.key.bytes),
            certificate: Signature<Organization>.fromBytes(bytes: orgPublicKey.certificate.bytes),
            notValidAfter: orgPublicKey.notValidAfter.date,
            now: orgPublicKey.notValidAfter.date.advanced(by: TimeInterval(-50))
        ))

        let journalistProvisioningPublicKey = try PublicKeysHelper
            .readLocalKeypairFile(path: "journalist_provisioning").publicKey
        let journalistProvisioningKeyObj = try XCTUnwrap(JournalistProvisioningKey(
            key: Sign.KeyPair.PublicKey(journalistProvisioningPublicKey.key.bytes),
            certificate: Signature<JournalistProvisioning>
                .fromBytes(bytes: journalistProvisioningPublicKey.certificate.bytes),
            signingKey: organizationSigningKey,
            notValidAfter: journalistProvisioningPublicKey.notValidAfter.date,
            now: journalistProvisioningPublicKey.notValidAfter.date.advanced(by: TimeInterval(-20))
        ))

        let journalistIdPublicKey = try PublicKeysHelper.readLocalKeypairFile(path: "journalist_id").publicKey
        let journoKey = JournalistIdPublicKey(
            key: Sign.KeyPair.PublicKey(journalistIdPublicKey.key.bytes),
            certificate: Signature<JournalistId>.fromBytes(bytes: journalistIdPublicKey.certificate.bytes),
            signingKey: journalistProvisioningKeyObj,
            notValidAfter: journalistIdPublicKey.notValidAfter.date,
            now: journalistIdPublicKey.notValidAfter.date.advanced(by: TimeInterval(-20))
        )

        XCTAssertEqual(journoKey?.key, Sign.KeyPair.PublicKey(journalistIdPublicKey.key.bytes))
        XCTAssertEqual(journoKey?.notValidAfter, journalistIdPublicKey.notValidAfter.date)
    }

    func testUnsuccessfulOrgKeyInitialization() throws {
        let orgPublicKey = try PublicKeysHelper.readLocalKeypairFile(path: "organization").publicKey

        // Corrupt the last byte of an otherwise-valid certificate so the signature fails to verify.
        var invalidCertificateBytes = orgPublicKey.certificate.bytes
        invalidCertificateBytes[invalidCertificateBytes.count - 1] ^= 0xFF

        XCTAssertNil(OrganizationPublicKey(
            key: Sign.KeyPair.PublicKey(orgPublicKey.key.bytes),
            certificate: Signature<Organization>.fromBytes(bytes: invalidCertificateBytes),
            notValidAfter: orgPublicKey.notValidAfter.date,
            now: orgPublicKey.notValidAfter.date.advanced(by: TimeInterval(-50))
        ))
    }

    func testUnsuccessfulInitialization() throws {
        let orgPublicKey = try PublicKeysHelper.readLocalKeypairFile(path: "organization").publicKey
        let organizationSigningKey = try XCTUnwrap(OrganizationPublicKey(
            key: Sign.KeyPair.PublicKey(orgPublicKey.key.bytes),
            certificate: Signature<Organization>.fromBytes(bytes: orgPublicKey.certificate.bytes),
            notValidAfter: orgPublicKey.notValidAfter.date,
            now: orgPublicKey.notValidAfter.date.advanced(by: TimeInterval(-50))
        ))

        // The journalist id key's certificate is signed by the journalist provisioning key, not the
        // organization key directly, so verifying it against `organizationSigningKey` must fail.
        let journalistIdPublicKey = try PublicKeysHelper.readLocalKeypairFile(path: "journalist_id").publicKey
        let journoKey = JournalistIdPublicKey(
            key: Sign.KeyPair.PublicKey(journalistIdPublicKey.key.bytes),
            certificate: Signature<JournalistId>.fromBytes(bytes: journalistIdPublicKey.certificate.bytes),
            signingKey: organizationSigningKey,
            notValidAfter: journalistIdPublicKey.notValidAfter.date,
            now: journalistIdPublicKey.notValidAfter.date.advanced(by: TimeInterval(-20))
        )

        XCTAssertTrue(journoKey == nil)
    }

    func testVerifyingOrganizationKeyFromTrustedRootKeys() throws {
        let orgPublicKey = try PublicKeysHelper.readLocalKeypairFile(path: "organization").publicKey
        let now = orgPublicKey.notValidAfter.date.advanced(by: TimeInterval(-50))

        let organizationSigningKey = try XCTUnwrap(OrganizationPublicKey(
            key: Sign.KeyPair.PublicKey(orgPublicKey.key.bytes),
            certificate: Signature<Organization>.fromBytes(bytes: orgPublicKey.certificate.bytes),
            notValidAfter: orgPublicKey.notValidAfter.date,
            now: now
        ))

        let trustedOrganizationKey = try XCTUnwrap(TrustedOrganizationPublicKey(
            key: Sign.KeyPair.PublicKey(orgPublicKey.key.bytes),
            certificate: Signature<TrustedOrganization>.fromBytes(bytes: orgPublicKey.certificate.bytes),
            notValidAfter: orgPublicKey.notValidAfter.date,
            now: now
        ))

        let verifiedOrgPk = VerifiedPublicKeysHierarchy.verifyOrganizationPublicKey(
            orgPk: organizationSigningKey,
            trustedOrgPks: [trustedOrganizationKey]
        )

        XCTAssertTrue(verifiedOrgPk?.key == organizationSigningKey.key)
    }

    func testVerifyingOrganizationKeyFailsIfNotInTrustedRootKeys() throws {
        let orgPublicKey = try PublicKeysHelper.readLocalKeypairFile(path: "organization").publicKey

        let organizationSigningKey = try XCTUnwrap(OrganizationPublicKey(
            key: Sign.KeyPair.PublicKey(orgPublicKey.key.bytes),
            certificate: Signature<Organization>.fromBytes(bytes: orgPublicKey.certificate.bytes),
            notValidAfter: orgPublicKey.notValidAfter.date,
            now: orgPublicKey.notValidAfter.date.advanced(by: TimeInterval(-50))
        ))

        XCTAssertNil(VerifiedPublicKeysHierarchy.verifyOrganizationPublicKey(
            orgPk: organizationSigningKey,
            // pass in an empty array of trust anchors
            trustedOrgPks: []
        ))
    }

    func testVerifiedKeys_whenTestDataAllCorrect_thenVerifies() throws {
        let config: StaticConfig = .devConfig
        let publicKeysData = try PublicKeysHelper.readLocalKeysFile()
        let trustedOrganizationPublicKeys = try PublicKeysHelper.readLocalTrustedOrganizationKeys(config: config)
        let currentTime = DateFunction.currentTime()

        _ = try VerifiedPublicKeys(
            publicKeysData: publicKeysData,
            trustedOrganizationPublicKeys: trustedOrganizationPublicKeys,
            currentTime: currentTime
        )
    }

    func testVerifiedKeys_whenEmptyTrustAnchor_thenThrows() throws {
        let publicKeysData = try PublicKeysHelper.readLocalKeysFile()
        let trustedOrganizationPublicKeys: [TrustedOrganizationPublicKey] = []
        let currentTime = DateFunction.currentTime()

        XCTAssertThrowsError(
            try VerifiedPublicKeys(
                publicKeysData: publicKeysData,
                trustedOrganizationPublicKeys: trustedOrganizationPublicKeys,
                currentTime: currentTime
            )
        ) { error in XCTAssertEqual(error as! VerificationError, VerificationError.missingTrustAnchor) }
    }

    func testVerifiedKeys_whenMinimalScenario_thenVerifies() throws {
        try verifyAndAssertMinimalScenario(scenario: .minimal)
    }

    /// The old vectors predate the `signature` field on identity keys; verification must succeed either way
    /// since the field is currently ignored.
    func testVerifiedKeys_whenMinimalScenarioWithoutIdKeySignature_thenVerifies() throws {
        try verifyAndAssertMinimalScenario(scenario: .minimalOldWithoutIdKeySignature)
    }

    private func verifyAndAssertMinimalScenario(scenario: IntegrationTestScenario) throws {
        let context = IntegrationTestScenarioContext(scenario: scenario)
        let publicKeysData = try context.loadKeys()
        let now = try context.loadNow()

        // Bypass loadKeysVerified's convenience wrapper since it always checks anchor expiry against the
        // shared, continuously-refreshed keys_generated_at.txt rather than this scenario's own vector dates.
        let trustedOrganizationKeys = try context.readTrustedOrganizationKeys(now: now)
        let verifiedKeys = try VerifiedPublicKeys(
            publicKeysData: publicKeysData,
            trustedOrganizationPublicKeys: trustedOrganizationKeys,
            currentTime: now
        )

        let keyHierarchy = try XCTUnwrap(verifiedKeys.verifiedHierarchies.first)
        XCTAssertFalse(keyHierarchy.organizationPublicKey.key.isEmpty)

        let coverNodeKeyHierarchy = try XCTUnwrap(keyHierarchy.verifiedCoverNodeKeyHierarchies.first)
        XCTAssertFalse(coverNodeKeyHierarchy.provisioningKey.key.isEmpty)
        XCTAssertEqual(coverNodeKeyHierarchy.idPublicKeys.count, 1)

        let journalists = keyHierarchy.allPublicKeysForJournalists()
        XCTAssertEqual(journalists.count, 1)

        for (_, journalistKeysGroups) in journalists {
            let journalistKeys = try XCTUnwrap(journalistKeysGroups.first)
            XCTAssertFalse(journalistKeys.id.key.isEmpty)
            XCTAssertFalse(journalistKeys.msg.isEmpty)
        }
    }
}
