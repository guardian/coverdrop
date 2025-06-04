@testable import CoverDropCore
import Sodium
import XCTest

// swiftlint:disable identifier_name line_length
final class KeyVerificationTests: XCTestCase {
    func testSuccessfullVerification() throws {
        let orgIdKey = "f9162ddd3609f1985b9d00c1701c2dfa046c819eefc81d5b3a8b6799c27827ee".hexStringToBytes()
        let certificate =
            "a05beac4862a73bc56243c91686bad92bf209131d34d0225f1c7832c96931f3cdeed011203ffe95a9fea74428735c22f2f3a8092ca65f1521192b38be8060d0c"
                .hexStringToBytes()!
        let notValidAfter = "2024-09-02T17:16:49.896447Z"

        let date = DateFormats.validateDate(date: notValidAfter)!

        let now = date.advanced(by: TimeInterval(-50))

        let organizationSigningKey = OrganizationPublicKey(
            key: Sign.KeyPair.PublicKey(orgIdKey!),
            certificate: Signature<Organization>.fromBytes(
                bytes: certificate
            ),
            notValidAfter: DateFormats.validateDate(date: notValidAfter)!,
            now: now
        )!

        let journalistProvisioningKey = "452bd05993423cff2e63dc625885d230c9b9cdfb7e703bc7671875d4d0d39fe9"
            .hexStringToBytes()

        let journalistProvisioningcertificate =
            "3c19fa5031f50156029d3f4ec54cf547009300095cc4258a05f736f055ddabd224d5d60e253e2e04d4fbba9c885b5d160e6282911ab32c685b7a04f9d0d7390a"
                .hexStringToBytes()
        let journalistProvisioningCertObj = Signature<JournalistProvisioning>
            .fromBytes(bytes: journalistProvisioningcertificate!)

        let journalistProvisioningCertNotValidAfter = "2024-02-19T17:16:50.908728Z"

        let journalistProvisioningCertNotValidAfterDate = DateFormats
            .validateDate(date: journalistProvisioningCertNotValidAfter)!

        let journalistProvisioningKeyObj = JournalistProvisioningKey(
            key: Sign.KeyPair.PublicKey(journalistProvisioningKey!),
            certificate: journalistProvisioningCertObj,
            signingKey: organizationSigningKey,
            notValidAfter: journalistProvisioningCertNotValidAfterDate,
            now: journalistProvisioningCertNotValidAfterDate.advanced(by: TimeInterval(-20))
        )

        let journalistIdKey = "a836d25394e1275fdb4256e9b37be79bd08fd68bf0ab07f573a9e8ae29681cef".hexStringToBytes()

        let journalistIdcertificate =
            "f1ee98fdbb51c72e8eeb53de091bf14a03bc34214cc7a0ef84b0c24683a979a697608db5117854e7b7e451496e28495c64650ea511e48068635f06060ba4ce02"
                .hexStringToBytes()
        let journalistIdCertObj = Signature<JournalistId>.fromBytes(bytes: journalistIdcertificate!)

        let journalistIdCertNotValidAfter = "2023-10-30T17:16:54.810680Z"

        let journalistIdCertNotValidAfterDate = DateFormats.validateDate(date: journalistIdCertNotValidAfter)!

        let journoKey = JournalistIdPublicKey(
            key: Sign.KeyPair.PublicKey(journalistIdKey!),
            certificate: journalistIdCertObj,
            signingKey: journalistProvisioningKeyObj!,
            notValidAfter: journalistIdCertNotValidAfterDate,
            now: journalistIdCertNotValidAfterDate.advanced(by: TimeInterval(-20))
        )

        XCTAssertTrue(journoKey != nil)
    }

    func testUnSuccessfullOrgKeyInitialisation() throws {
        let orgIdKey = "f9162ddd3609f1985b9d00c1701c2dfa046c819eefc81d5b3a8b6799c27827ee".hexStringToBytes()
        let certificate =
            "a05beac4862a73bc56243c91686bad92bf209131d34d0225f1c7832c96931f3cdeed011203ffe95a9fea74428735c22f2f3a8092ca65f1521192b38be8060d0d"
                .hexStringToBytes()!
        let notValidAfter = "2024-09-02T17:16:49.896447Z"

        let date = DateFormats.validateDate(date: notValidAfter)!

        let now = date.advanced(by: TimeInterval(-50))

        XCTAssertNil(OrganizationPublicKey(
            key: Sign.KeyPair.PublicKey(orgIdKey!),
            certificate: Signature<Organization>.fromBytes(
                bytes: certificate
            ),
            notValidAfter: DateFormats.validateDate(date: notValidAfter)!,
            now: now
        ))
    }

    func testUnSuccessfullInitialisation() throws {
        let orgIdKey = "f9162ddd3609f1985b9d00c1701c2dfa046c819eefc81d5b3a8b6799c27827ee".hexStringToBytes()
        let certificate =
            "a05beac4862a73bc56243c91686bad92bf209131d34d0225f1c7832c96931f3cdeed011203ffe95a9fea74428735c22f2f3a8092ca65f1521192b38be8060d0c"
                .hexStringToBytes()!
        let notValidAfter = "2024-09-02T17:16:49.896447Z"

        let date = DateFormats.validateDate(date: notValidAfter)!

        let now = date.advanced(by: TimeInterval(-50))

        let organizationSigningKey = OrganizationPublicKey(
            key: Sign.KeyPair.PublicKey(orgIdKey!),
            certificate: Signature<Organization>.fromBytes(
                bytes: certificate
            ),
            notValidAfter: DateFormats.validateDate(date: notValidAfter)!,
            now: now
        )!

        let journalistIdKey = "c098c0ee644534b310683165dee4f1551081cad1ee579acf2dc3c277144d97fa".hexStringToBytes()

        let journalistIdcertificate =
            "615094f7fbb47dbdde1f97a8197388ed0005a20a438be93a2462dd1c4c8a668a4381004f8d97f6de20739ade7d60a67487c193511dd5a0b86a2baff94822990b"
                .hexStringToBytes()
        let journalistIdCertObj = Signature<JournalistId>.fromBytes(bytes: journalistIdcertificate!)

        let journoKey = JournalistIdPublicKey(
            key: Sign.KeyPair.PublicKey(journalistIdKey!),
            certificate: journalistIdCertObj,
            signingKey: organizationSigningKey,
            notValidAfter: DateFunction.currentTime(),
            now: DateFunction.currentTime()
        )

        XCTAssertTrue(journoKey == nil)
    }

    func testVerifyingOrganizationKeyFromTrustedRootKeys() throws {
        let orgIdKey = "f9162ddd3609f1985b9d00c1701c2dfa046c819eefc81d5b3a8b6799c27827ee".hexStringToBytes()
        let certificate =
            "a05beac4862a73bc56243c91686bad92bf209131d34d0225f1c7832c96931f3cdeed011203ffe95a9fea74428735c22f2f3a8092ca65f1521192b38be8060d0c"
                .hexStringToBytes()!
        let notValidAfter = "2024-09-02T17:16:49.896447Z"

        let date = DateFormats.validateDate(date: notValidAfter)!

        let now = date.advanced(by: TimeInterval(-50))

        let organizationSigningKey = OrganizationPublicKey(
            key: Sign.KeyPair.PublicKey(orgIdKey!),
            certificate: Signature<Organization>.fromBytes(
                bytes: certificate
            ),
            notValidAfter: DateFormats.validateDate(date: notValidAfter)!,
            now: now
        )!

        let organizationSigningKey1 = TrustedOrganizationPublicKey(
            key: Sign.KeyPair.PublicKey(orgIdKey!),
            certificate: Signature<TrustedOrganization>.fromBytes(
                bytes: certificate
            ),
            notValidAfter: DateFormats.validateDate(date: notValidAfter)!,
            now: now
        )!

        let trustedOrgPks = [organizationSigningKey1]

        let verifiedOrgPk = VerifiedPublicKeysHierarchy.verifyOrganizationPublicKey(
            orgPk: organizationSigningKey,
            trustedOrgPks: trustedOrgPks
        )

        XCTAssertTrue(verifiedOrgPk!.key == organizationSigningKey.key)
    }

    func testVerifyingOrganizationKeyFailsIfNotInTrustedRootKeys() throws {
        let orgIdKey = "f9162ddd3609f1985b9d00c1701c2dfa046c819eefc81d5b3a8b6799c27827ee".hexStringToBytes()
        let certificate =
            "a05beac4862a73bc56243c91686bad92bf209131d34d0225f1c7832c96931f3cdeed011203ffe95a9fea74428735c22f2f3a8092ca65f1521192b38be8060d0c"
                .hexStringToBytes()!
        let notValidAfter = "2024-09-02T17:16:49.896447Z"

        let date = DateFormats.validateDate(date: notValidAfter)!

        let now = date.advanced(by: TimeInterval(-50))

        let organizationSigningKey = OrganizationPublicKey(
            key: Sign.KeyPair.PublicKey(orgIdKey!),
            certificate: Signature<Organization>.fromBytes(
                bytes: certificate
            ),
            notValidAfter: DateFormats.validateDate(date: notValidAfter)!,
            now: now
        )!

        let trustedOrgPks: [TrustedOrganizationPublicKey] = []

        XCTAssertNil(VerifiedPublicKeysHierarchy.verifyOrganizationPublicKey(
            orgPk: organizationSigningKey,
            trustedOrgPks: trustedOrgPks
        ))
    }
}

// swiftlint:enable identifier_name line_length
