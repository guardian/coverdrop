@testable import CoverDropCore
import Sodium
import XCTest

final class KeyVerificationTests: XCTestCase {
    // MARK: - legacy certificate checks

    // Corrupt the legacy certificate or use the wrong signer; independent of the new signature check.

    func testSuccessfulVerification() throws {
        let orgPublicKey = try PublicKeysHelper.readLocalKeypairFile(path: "organization").publicKey
        let organizationSigningKey = try XCTUnwrap(OrganizationPublicKey(
            key: Sign.KeyPair.PublicKey(orgPublicKey.key.bytes),
            certificate: Signature<Organization>.fromBytes(bytes: orgPublicKey.certificate.bytes),
            signature: Signature<Organization>.fromBytes(bytes: orgPublicKey.certificate.bytes),
            notValidAfter: orgPublicKey.notValidAfter.date,
            now: orgPublicKey.notValidAfter.date.advanced(by: TimeInterval(-50))
        ))

        let journalistProvisioningPublicKey = try PublicKeysHelper
            .readLocalKeypairFile(path: "journalist_provisioning").publicKey
        let journalistProvisioningKeyObj = try XCTUnwrap(JournalistProvisioningKey(
            key: Sign.KeyPair.PublicKey(journalistProvisioningPublicKey.key.bytes),
            certificate: Signature<JournalistProvisioning>
                .fromBytes(bytes: journalistProvisioningPublicKey.certificate.bytes),
            signature: journalistProvisioningPublicKey.signature.map {
                Signature<JournalistProvisioning>.fromBytes(bytes: $0.bytes)
            },
            signingKey: organizationSigningKey,
            notValidAfter: journalistProvisioningPublicKey.notValidAfter.date,
            now: journalistProvisioningPublicKey.notValidAfter.date.advanced(by: TimeInterval(-20))
        ))

        let journalistIdPublicKey = try PublicKeysHelper.readLocalKeypairFile(path: "journalist_id").publicKey
        let journoKey = JournalistIdPublicKey(
            key: Sign.KeyPair.PublicKey(journalistIdPublicKey.key.bytes),
            certificate: Signature<JournalistId>.fromBytes(bytes: journalistIdPublicKey.certificate.bytes),
            signature: journalistIdPublicKey.signature.map { Signature<JournalistId>.fromBytes(bytes: $0.bytes) },
            signingKey: journalistProvisioningKeyObj,
            identity: "journalist_id",
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
            signature: orgPublicKey.signature.map { Signature<Organization>.fromBytes(bytes: $0.bytes) },
            notValidAfter: orgPublicKey.notValidAfter.date,
            now: orgPublicKey.notValidAfter.date.advanced(by: TimeInterval(-50))
        ))
    }

    func testUnsuccessfulInitialization() throws {
        let orgPublicKey = try PublicKeysHelper.readLocalKeypairFile(path: "organization").publicKey
        let organizationSigningKey = try XCTUnwrap(OrganizationPublicKey(
            key: Sign.KeyPair.PublicKey(orgPublicKey.key.bytes),
            certificate: Signature<Organization>.fromBytes(bytes: orgPublicKey.certificate.bytes),
            signature: orgPublicKey.signature.map { Signature<Organization>.fromBytes(bytes: $0.bytes) },
            notValidAfter: orgPublicKey.notValidAfter.date,
            now: orgPublicKey.notValidAfter.date.advanced(by: TimeInterval(-50))
        ))

        // The journalist id key's certificate is signed by the journalist provisioning key, not the
        // organization key directly, so verifying it against `organizationSigningKey` must fail.
        let journalistIdPublicKey = try PublicKeysHelper.readLocalKeypairFile(path: "journalist_id").publicKey
        let journoKey = JournalistIdPublicKey(
            key: Sign.KeyPair.PublicKey(journalistIdPublicKey.key.bytes),
            certificate: Signature<JournalistId>.fromBytes(bytes: journalistIdPublicKey.certificate.bytes),
            signature: journalistIdPublicKey.signature.map { Signature<JournalistId>.fromBytes(bytes: $0.bytes) },
            signingKey: organizationSigningKey,
            identity: "journalist_id",
            notValidAfter: journalistIdPublicKey.notValidAfter.date,
            now: journalistIdPublicKey.notValidAfter.date.advanced(by: TimeInterval(-20))
        )

        XCTAssertTrue(journoKey == nil)
    }

    // MARK: - signature checks

    // Using  `minimal_scenario` vectors with valid certificates, signers, and expiry times, only ever
    // corrupt the `signature` field, or the identity/role it is verified against, so they can only pass
    // if that new check is actually enforced.

    private struct VerifiedScenario {
        let publicKeysData: PublicKeysData
        let verifiedKeys: VerifiedPublicKeys
        let now: Date
    }

    private func loadVerifiedMinimalScenario(
        scenario: IntegrationTestScenario = .minimal
    ) throws -> VerifiedScenario {
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
        return VerifiedScenario(publicKeysData: publicKeysData, verifiedKeys: verifiedKeys, now: now)
    }

    /// Flips the last byte of `signature`, leaving the legacy `certificate` untouched.
    private func corruptSignature(
        of key: UnverifiedSignedPublicSigningKeyData
    ) throws -> UnverifiedSignedPublicSigningKeyData {
        var signatureBytes = try XCTUnwrap(key.signature).bytes
        signatureBytes[signatureBytes.count - 1] ^= 0xFF
        return UnverifiedSignedPublicSigningKeyData(
            key: key.key,
            certificate: key.certificate,
            signature: HexEncodedString(bytes: signatureBytes),
            notValidAfter: key.notValidAfter
        )
    }

    /// Flips the last byte of `signature`, leaving the legacy `certificate` untouched.
    private func corruptSignature(
        of key: UnverifiedSignedPublicEncryptionKeyData
    ) throws -> UnverifiedSignedPublicEncryptionKeyData {
        var signatureBytes = try XCTUnwrap(key.signature).bytes
        signatureBytes[signatureBytes.count - 1] ^= 0xFF
        return UnverifiedSignedPublicEncryptionKeyData(
            key: key.key,
            certificate: key.certificate,
            signature: HexEncodedString(bytes: signatureBytes),
            notValidAfter: key.notValidAfter
        )
    }

    func testSigningKeySignatureCorruption_whenOnlySignatureCorrupted_thenRejected() throws {
        let scenario = try loadVerifiedMinimalScenario()
        let publicKeysData = scenario.publicKeysData
        let now = scenario.now
        let keyHierarchy = try XCTUnwrap(publicKeysData.keys.first)

        let validOrgKey = keyHierarchy.organizationPublicKey
        let orgKeyWithCorruptedSignature = try corruptSignature(of: validOrgKey)

        XCTAssertNil(OrganizationPublicKey(
            key: Sign.KeyPair.PublicKey(validOrgKey.key.bytes),
            certificate: Signature<Organization>.fromBytes(bytes: validOrgKey.certificate.bytes),
            signature: orgKeyWithCorruptedSignature.signature
                .map { Signature<Organization>.fromBytes(bytes: $0.bytes) },
            notValidAfter: validOrgKey.notValidAfter.date,
            now: now
        ))
    }

    func testJournalistProvisioningKeySignatureCorruption_whenOnlySignatureCorrupted_thenRejected() throws {
        let scenario = try loadVerifiedMinimalScenario()
        let publicKeysData = scenario.publicKeysData
        let verifiedKeys = scenario.verifiedKeys
        let now = scenario.now
        let keyHierarchy = try XCTUnwrap(publicKeysData.keys.first)
        let organizationKey = try XCTUnwrap(verifiedKeys.verifiedHierarchies.first?.organizationPublicKey)
        let provisioningKeyData = try XCTUnwrap(keyHierarchy.journalists.first?.provisioningPublicKey)
        let corruptedProvisioningKey = try corruptSignature(of: provisioningKeyData)

        // Explicit return type needed: `R` only appears in `fromUnverified`'s return type.
        func verify(_ key: UnverifiedSignedPublicSigningKeyData) throws -> JournalistProvisioningKey {
            try JournalistProvisioningKey.fromUnverified(unverifiedKey: key, signingKey: organizationKey, now: now)
        }

        // Control: the untouched key verifies, so the failure below is down to the corrupted signature.
        XCTAssertNoThrow(try verify(provisioningKeyData))
        XCTAssertThrowsError(try verify(corruptedProvisioningKey))
    }

    func testCoverNodeProvisioningKeySignatureCorruption_whenOnlySignatureCorrupted_thenRejected() throws {
        let scenario = try loadVerifiedMinimalScenario()
        let publicKeysData = scenario.publicKeysData
        let verifiedKeys = scenario.verifiedKeys
        let now = scenario.now
        let keyHierarchy = try XCTUnwrap(publicKeysData.keys.first)
        let organizationKey = try XCTUnwrap(verifiedKeys.verifiedHierarchies.first?.organizationPublicKey)
        let provisioningKeyData = try XCTUnwrap(keyHierarchy.coverNodes.first?.provisioningPublicKey)
        let corruptedProvisioningKey = try corruptSignature(of: provisioningKeyData)

        // Explicit return type needed: `R` only appears in `fromUnverified`'s return type.
        func verify(_ key: UnverifiedSignedPublicSigningKeyData) throws -> CoverNodeProvisioningKey {
            try CoverNodeProvisioningKey.fromUnverified(unverifiedKey: key, signingKey: organizationKey, now: now)
        }

        // Control: the untouched key verifies, so the failure below is down to the corrupted signature.
        XCTAssertNoThrow(try verify(provisioningKeyData))
        XCTAssertThrowsError(try verify(corruptedProvisioningKey))
    }

    func testIdentityKeySignatureCorruption_whenOnlySignatureCorrupted_thenRejected() throws {
        let scenario = try loadVerifiedMinimalScenario()
        let publicKeysData = scenario.publicKeysData
        let verifiedKeys = scenario.verifiedKeys
        let now = scenario.now
        let keyHierarchy = try XCTUnwrap(publicKeysData.keys.first)
        let journalistProvisioningKey = try XCTUnwrap(
            verifiedKeys.verifiedHierarchies.first?.verifiedJournalistKeyHierarchies.first?.provisioningKey
        )
        let journalistIdKeyData = try XCTUnwrap(
            keyHierarchy.journalists.first?.journalists["static_test_journalist"]?.first?.idPk
        )
        let corruptedIdKey = try corruptSignature(of: journalistIdKeyData)

        XCTAssertThrowsError(try JournalistIdPublicKey.fromUnverified(
            unverifiedKey: corruptedIdKey,
            signingKey: journalistProvisioningKey,
            identity: "static_test_journalist",
            now: now
        ))
    }

    func testEncryptionKeySignatureCorruption_whenOnlySignatureCorrupted_thenRejected() throws {
        let scenario = try loadVerifiedMinimalScenario()
        let publicKeysData = scenario.publicKeysData
        let verifiedKeys = scenario.verifiedKeys
        let now = scenario.now
        let keyHierarchy = try XCTUnwrap(publicKeysData.keys.first)
        let journalistIdKey = try XCTUnwrap(
            verifiedKeys.verifiedHierarchies.first?.verifiedJournalistKeyHierarchies.first?
                .keys["static_test_journalist"]?.first?.id
        )
        let messageKeyData = try XCTUnwrap(
            keyHierarchy.journalists.first?.journalists["static_test_journalist"]?.first?.msgPks.first
        )
        let corruptedMessageKey = try corruptSignature(of: messageKeyData)

        // Explicit return type needed: `R` only appears in `fromUnverified`'s return type, so it can't be
        // inferred from the arguments alone.
        func verify() throws -> JournalistMessagingPublicKey {
            try JournalistMessagingPublicKey.fromUnverified(
                unverifiedMessageKey: corruptedMessageKey,
                signingKey: journalistIdKey,
                now: now
            )
        }

        XCTAssertThrowsError(try verify())
    }

    func testJournalistIdentitySubstitution_whenVerifiedAgainstDifferentDictionaryIdentity_thenRejected() throws {
        let scenario = try loadVerifiedMinimalScenario()
        let publicKeysData = scenario.publicKeysData
        let verifiedKeys = scenario.verifiedKeys
        let now = scenario.now
        let keyHierarchy = try XCTUnwrap(publicKeysData.keys.first)
        let journalistProvisioningKey = try XCTUnwrap(
            verifiedKeys.verifiedHierarchies.first?.verifiedJournalistKeyHierarchies.first?.provisioningKey
        )
        // Untouched, validly-signed id key for "static_test_journalist".
        let journalistIdKeyData = try XCTUnwrap(
            keyHierarchy.journalists.first?.journalists["static_test_journalist"]?.first?.idPk
        )

        // Simulates the published-keys dictionary pointing a different journalist id at this id key.
        XCTAssertThrowsError(try JournalistIdPublicKey.fromUnverified(
            unverifiedKey: journalistIdKeyData,
            signingKey: journalistProvisioningKey,
            identity: "a_different_journalist",
            now: now
        ))
    }

    func testCoverNodeIdentitySubstitution_whenVerifiedAgainstDifferentDictionaryIdentity_thenRejected() throws {
        let scenario = try loadVerifiedMinimalScenario()
        let publicKeysData = scenario.publicKeysData
        let verifiedKeys = scenario.verifiedKeys
        let now = scenario.now
        let keyHierarchy = try XCTUnwrap(publicKeysData.keys.first)
        let coverNodeProvisioningKey = try XCTUnwrap(
            verifiedKeys.verifiedHierarchies.first?.verifiedCoverNodeKeyHierarchies.first?.provisioningKey
        )
        // Untouched, validly-signed id key for "covernode_001".
        let coverNodeIdKeyData = try XCTUnwrap(
            keyHierarchy.coverNodes.first?.covernodes["covernode_001"]?.first?.idPk
        )

        // Simulates the published-keys dictionary pointing a different covernode instance at this id key.
        XCTAssertThrowsError(try CoverNodeIdPublicKey.fromUnverified(
            unverifiedKey: coverNodeIdKeyData,
            signingKey: coverNodeProvisioningKey,
            identity: "covernode_666",
            now: now
        ))
    }

    func testCoverNodeIdKeyRoleSubstitution_whenVerifiedAsJournalistIdentity_thenRejected() throws {
        let scenario = try loadVerifiedMinimalScenario()
        let publicKeysData = scenario.publicKeysData
        let verifiedKeys = scenario.verifiedKeys
        let now = scenario.now
        let keyHierarchy = try XCTUnwrap(publicKeysData.keys.first)
        let coverNodeProvisioningKey = try XCTUnwrap(
            verifiedKeys.verifiedHierarchies.first?.verifiedCoverNodeKeyHierarchies.first?.provisioningKey
        )
        // Same key material, signer, and identity as the valid covernode id key - only the role (bound via
        // `JournalistId` here, instead of the actual `CoverNodeId`) differs from what was signed.
        let coverNodeIdKeyData = try XCTUnwrap(
            keyHierarchy.coverNodes.first?.covernodes["covernode_001"]?.first?.idPk
        )

        XCTAssertThrowsError(try SignedIdentityPublicSigningKey<JournalistId, CoverNodeProvisioningKey>.fromUnverified(
            unverifiedKey: coverNodeIdKeyData,
            signingKey: coverNodeProvisioningKey,
            identity: "covernode_001",
            now: now
        ))
    }

    // MARK: - test key expiry

    // The certificates in these keys are valid, so only `now` relative to `notValidAfter` decides the outcome.
    // A key is still valid at exactly `notValidAfter` and expired one second later.

    private let oneSecond = TimeInterval(1)

    // The parent keys are loaded shortly before their own expiry so that they are valid signers.

    private func loadOrganizationKey() throws -> OrganizationPublicKey {
        let notValidAfter = try PublicKeysHelper.readLocalKeypairFile(path: "organization").publicKey.notValidAfter
        return try XCTUnwrap(makeOrganizationKey(now: notValidAfter.date.advanced(by: -oneSecond)))
    }

    private func loadJournalistProvisioningKey() throws -> JournalistProvisioningKey {
        let notValidAfter = try PublicKeysHelper.readLocalKeypairFile(path: "journalist_provisioning").publicKey
            .notValidAfter
        return try XCTUnwrap(makeJournalistProvisioningKey(now: notValidAfter.date.advanced(by: -oneSecond)))
    }

    private func loadJournalistIdKey() throws -> JournalistIdPublicKey {
        let notValidAfter = try PublicKeysHelper.readLocalKeypairFile(path: "journalist_id").publicKey.notValidAfter
        return try XCTUnwrap(makeJournalistIdKey(now: notValidAfter.date.advanced(by: -oneSecond)))
    }

    private func makeOrganizationKey(now: Date) throws -> OrganizationPublicKey? {
        let data = try PublicKeysHelper.readLocalKeypairFile(path: "organization").publicKey
        return OrganizationPublicKey(
            key: Sign.KeyPair.PublicKey(data.key.bytes),
            certificate: Signature<Organization>.fromBytes(bytes: data.certificate.bytes),
            signature: data.signature.map { Signature<Organization>.fromBytes(bytes: $0.bytes) },
            notValidAfter: data.notValidAfter.date,
            now: now
        )
    }

    private func makeJournalistProvisioningKey(now: Date) throws -> JournalistProvisioningKey? {
        let data = try PublicKeysHelper.readLocalKeypairFile(path: "journalist_provisioning").publicKey
        let organizationKey = try loadOrganizationKey()
        return JournalistProvisioningKey(
            key: Sign.KeyPair.PublicKey(data.key.bytes),
            certificate: Signature<JournalistProvisioning>.fromBytes(bytes: data.certificate.bytes),
            signature: data.signature.map { Signature<JournalistProvisioning>.fromBytes(bytes: $0.bytes) },
            signingKey: organizationKey,
            notValidAfter: data.notValidAfter.date,
            now: now
        )
    }

    private func makeJournalistIdKey(now: Date) throws -> JournalistIdPublicKey? {
        let data = try PublicKeysHelper.readLocalKeypairFile(path: "journalist_id").publicKey
        let provisioningKey = try loadJournalistProvisioningKey()
        return JournalistIdPublicKey(
            key: Sign.KeyPair.PublicKey(data.key.bytes),
            certificate: Signature<JournalistId>.fromBytes(bytes: data.certificate.bytes),
            signature: data.signature.map { Signature<JournalistId>.fromBytes(bytes: $0.bytes) },
            signingKey: provisioningKey,
            identity: "journalist_id",
            notValidAfter: data.notValidAfter.date,
            now: now
        )
    }

    private func makeJournalistMessagingKey(now: Date) throws -> JournalistMessagingPublicKey? {
        let data = try PublicKeysHelper.readLocalKeypairFile(path: "journalist_msg").publicKey
        let idKey = try loadJournalistIdKey()
        return JournalistMessagingPublicKey(
            key: PublicEncryptionKey<JournalistMessaging>(key: Box.KeyPair.PublicKey(data.key.bytes)),
            certificate: Signature<JournalistMessaging>.fromBytes(bytes: data.certificate.bytes),
            signature: data.signature.map { Signature<JournalistMessaging>.fromBytes(bytes: $0.bytes) },
            signingKey: idKey,
            notValidAfter: data.notValidAfter.date,
            now: now
        )
    }

    func testExpiredOrgKeyInitialization() throws {
        let notValidAfter = try PublicKeysHelper.readLocalKeypairFile(path: "organization").publicKey
            .notValidAfter.date

        XCTAssertNotNil(try makeOrganizationKey(now: notValidAfter))
        XCTAssertNil(try makeOrganizationKey(now: notValidAfter.advanced(by: oneSecond)))
    }

    func testExpiredProvisioningKeyInitialization() throws {
        let notValidAfter = try PublicKeysHelper.readLocalKeypairFile(path: "journalist_provisioning").publicKey
            .notValidAfter.date

        XCTAssertNotNil(try makeJournalistProvisioningKey(now: notValidAfter))
        XCTAssertNil(try makeJournalistProvisioningKey(now: notValidAfter.advanced(by: oneSecond)))
    }

    func testExpiredIdKeyInitialization() throws {
        let notValidAfter = try PublicKeysHelper.readLocalKeypairFile(path: "journalist_id").publicKey
            .notValidAfter.date

        XCTAssertNotNil(try makeJournalistIdKey(now: notValidAfter))
        XCTAssertNil(try makeJournalistIdKey(now: notValidAfter.advanced(by: oneSecond)))
    }

    func testExpiredMessagingKeyInitialization() throws {
        let notValidAfter = try PublicKeysHelper.readLocalKeypairFile(path: "journalist_msg").publicKey
            .notValidAfter.date

        XCTAssertNotNil(try makeJournalistMessagingKey(now: notValidAfter))
        XCTAssertNil(try makeJournalistMessagingKey(now: notValidAfter.advanced(by: oneSecond)))
    }

    // MARK: - verifying the key hierarchy

    func testVerifyingOrganizationKeyFromTrustedRootKeys() throws {
        let orgPublicKey = try PublicKeysHelper.readLocalKeypairFile(path: "organization").publicKey
        let now = orgPublicKey.notValidAfter.date.advanced(by: TimeInterval(-50))

        let organizationSigningKey = try XCTUnwrap(OrganizationPublicKey(
            key: Sign.KeyPair.PublicKey(orgPublicKey.key.bytes),
            certificate: Signature<Organization>.fromBytes(bytes: orgPublicKey.certificate.bytes),
            signature: orgPublicKey.signature.map { Signature<Organization>.fromBytes(bytes: $0.bytes) },
            notValidAfter: orgPublicKey.notValidAfter.date,
            now: now
        ))

        let trustedOrganizationKey = try XCTUnwrap(TrustedOrganizationPublicKey(
            key: Sign.KeyPair.PublicKey(orgPublicKey.key.bytes),
            certificate: Signature<TrustedOrganization>.fromBytes(bytes: orgPublicKey.certificate.bytes),
            signature: orgPublicKey.signature.map { Signature<TrustedOrganization>.fromBytes(bytes: $0.bytes) },
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
            signature: orgPublicKey.signature.map { Signature<Organization>.fromBytes(bytes: $0.bytes) },
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
        let verifiedKeys = try loadVerifiedMinimalScenario(scenario: scenario).verifiedKeys

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
