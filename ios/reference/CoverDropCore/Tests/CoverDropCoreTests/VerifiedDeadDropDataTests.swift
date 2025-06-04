@testable import CoverDropCore
import XCTest

final class VerifiedDeadDropDataTests: XCTestCase {
    func testVerification_happyPath() throws {
        let testContext = IntegrationTestScenarioContext(scenario: .minimal)
        let verifiedKeys = try testContext.loadKeysVerified()
        let deadDropData = try testContext.loadDeadDrop()
        let result = VerifiedDeadDrops.fromAllDeadDropData(deadDrops: deadDropData, verifiedKeys: verifiedKeys)
        XCTAssertEqual(result.deadDrops.map { $0.id }, [1, 2, 3])
    }

    func testVerification_whenDateManipulated_thenSkipped() throws {
        let testContext = IntegrationTestScenarioContext(scenario: .minimal)
        let verifiedKeys = try testContext.loadKeysVerified()
        var deadDropData = try testContext.loadDeadDrop()

        // manipulate the date
        deadDropData.deadDrops[1].createdAt =
            try RFC3339DateTimeString(date: deadDropData.deadDrops[1].createdAt.date.minusSeconds(1))

        let result = VerifiedDeadDrops.fromAllDeadDropData(deadDrops: deadDropData, verifiedKeys: verifiedKeys)
        XCTAssertEqual(result.deadDrops.map { $0.id }, [1, 3])
    }

    func testVerification_whenDataManipulated_thenSkipped() throws {
        let testContext = IntegrationTestScenarioContext(scenario: .minimal)
        let verifiedKeys = try testContext.loadKeysVerified()
        var deadDropData = try testContext.loadDeadDrop()

        // manipulate the data
        deadDropData.deadDrops[1].data.bytes[0] = deadDropData.deadDrops[1].data.bytes[0] ^ 0x01

        let result = VerifiedDeadDrops.fromAllDeadDropData(deadDrops: deadDropData, verifiedKeys: verifiedKeys)
        XCTAssertEqual(result.deadDrops.map { $0.id }, [1, 3])
    }

    func testVerification_whenSignatureManipulated_thenSkipped() throws {
        let testContext = IntegrationTestScenarioContext(scenario: .minimal)
        let verifiedKeys = try testContext.loadKeysVerified()
        var deadDropData = try testContext.loadDeadDrop()

        // manipulate the signature
        deadDropData.deadDrops[1].signature!.bytes[0] = deadDropData.deadDrops[1].signature!.bytes[0] ^ 0x01

        let result = VerifiedDeadDrops.fromAllDeadDropData(deadDrops: deadDropData, verifiedKeys: verifiedKeys)
        XCTAssertEqual(result.deadDrops.map { $0.id }, [1, 3])
    }

    /// This test case is correct, as we are not relying on the `cert` for when the signature looks meaningful.
    func testVerification_whenCertManipulated_thenPasses() throws {
        let testContext = IntegrationTestScenarioContext(scenario: .minimal)
        let verifiedKeys = try testContext.loadKeysVerified()
        var deadDropData = try testContext.loadDeadDrop()

        // manipulate the cert
        deadDropData.deadDrops[1].cert!.bytes[0] = deadDropData.deadDrops[1].cert!.bytes[0] ^ 0x01

        let result = VerifiedDeadDrops.fromAllDeadDropData(deadDrops: deadDropData, verifiedKeys: verifiedKeys)
        XCTAssertEqual(result.deadDrops.map { $0.id }, [1, 2, 3])
    }

    func testVerification_whenSignatureNotMeaningful_andCertInvalid_thenSkipped() throws {
        let testContext = IntegrationTestScenarioContext(scenario: .minimal)
        let verifiedKeys = try testContext.loadKeysVerified()
        var deadDropData = try testContext.loadDeadDrop()

        // set the signature to be just 0x00 bytes
        deadDropData.deadDrops[1].signature = HexEncodedString(bytes: [0x00, 0x00, 0x00])

        // manipulate the cert
        deadDropData.deadDrops[1].cert!.bytes[0] = deadDropData.deadDrops[1].cert!.bytes[0] ^ 0x01

        let result = VerifiedDeadDrops.fromAllDeadDropData(deadDrops: deadDropData, verifiedKeys: verifiedKeys)
        XCTAssertEqual(result.deadDrops.map { $0.id }, [1, 3])
    }

    func testVerification_whenSignatureIsNil_andCertIsNil_thenSkipped() throws {
        let testContext = IntegrationTestScenarioContext(scenario: .minimal)
        let verifiedKeys = try testContext.loadKeysVerified()
        var deadDropData = try testContext.loadDeadDrop()

        // set the signature to be nil
        deadDropData.deadDrops[1].signature = nil

        // manipulate the cert
        deadDropData.deadDrops[1].cert = nil

        let result = VerifiedDeadDrops.fromAllDeadDropData(deadDrops: deadDropData, verifiedKeys: verifiedKeys)
        XCTAssertEqual(result.deadDrops.map { $0.id }, [1, 3])
    }

    func testVerification_withLegacy_happyPath() throws {
        let testContext = IntegrationTestScenarioContext(scenario: .minimalLegacy)
        let verifiedKeys = try testContext.loadKeysVerified()
        let deadDropData = try testContext.loadDeadDrop()
        let result = VerifiedDeadDrops.fromAllDeadDropData(deadDrops: deadDropData, verifiedKeys: verifiedKeys)
        XCTAssertEqual(result.deadDrops.map { $0.id }, [1, 2, 3])
    }

    /// This test case is correct and the reason why we are migrating away from the legacy signature (see #2998(.
    func testVerification_withLegacy_whenDateManipulated_thenPasses() throws {
        let testContext = IntegrationTestScenarioContext(scenario: .minimalLegacy)
        let verifiedKeys = try testContext.loadKeysVerified()
        var deadDropData = try testContext.loadDeadDrop()

        // manipulate the date (this should not have an effect with legacy `cert` since it is not covered)
        deadDropData.deadDrops[1].createdAt =
            try RFC3339DateTimeString(date: deadDropData.deadDrops[1].createdAt.date.minusSeconds(1))

        let result = VerifiedDeadDrops.fromAllDeadDropData(deadDrops: deadDropData, verifiedKeys: verifiedKeys)
        XCTAssertEqual(result.deadDrops.map { $0.id }, [1, 2, 3])
    }

    func testVerification_withLegacy_whenDataManipulated_thenSkipped() throws {
        let testContext = IntegrationTestScenarioContext(scenario: .minimalLegacy)
        let verifiedKeys = try testContext.loadKeysVerified()
        var deadDropData = try testContext.loadDeadDrop()

        // manipulate the date
        deadDropData.deadDrops[1].data.bytes[0] = deadDropData.deadDrops[1].data.bytes[0] ^ 0x01

        let result = VerifiedDeadDrops.fromAllDeadDropData(deadDrops: deadDropData, verifiedKeys: verifiedKeys)
        XCTAssertEqual(result.deadDrops.map { $0.id }, [1, 3])
    }

    func testVerification_withLegacy_whenCertManipulated_thenSkipped() throws {
        let testContext = IntegrationTestScenarioContext(scenario: .minimalLegacy)
        let verifiedKeys = try testContext.loadKeysVerified()
        var deadDropData = try testContext.loadDeadDrop()

        // manipulate the cert
        deadDropData.deadDrops[1].cert!.bytes[0] = deadDropData.deadDrops[1].cert!.bytes[0] ^ 0x01

        let result = VerifiedDeadDrops.fromAllDeadDropData(deadDrops: deadDropData, verifiedKeys: verifiedKeys)
        XCTAssertEqual(result.deadDrops.map { $0.id }, [1, 3])
    }

    func testVerification_withLegacy_whenTimeIsFarInTheFuture_thenSkipped() throws {
        let testContext = IntegrationTestScenarioContext(scenario: .minimalLegacy)
        let verifiedKeys = try testContext.loadKeysVerified()
        var deadDropData = try testContext.loadDeadDrop()

        // manipulate the date
        let oneHourAgo = try DateFunction.currentTime().minusSeconds(3600)
        let oneHourAhead = try DateFunction.currentTime().plusSeconds(3600)
        let twoWeeksAhead = try DateFunction.currentTime().plusSeconds(14 * 24 * 3600)
        deadDropData.deadDrops[0].createdAt = RFC3339DateTimeString(date: oneHourAgo)
        deadDropData.deadDrops[1].createdAt = RFC3339DateTimeString(date: oneHourAhead)
        deadDropData.deadDrops[2].createdAt = RFC3339DateTimeString(date: twoWeeksAhead)

        let result = VerifiedDeadDrops.fromAllDeadDropData(deadDrops: deadDropData, verifiedKeys: verifiedKeys)
        XCTAssertEqual(result.deadDrops.map { $0.id }, [1, 2])
    }

    func testIsMeaningfulSignature() throws {
        let nullSignature: HexEncodedString? = nil
        XCTAssertFalse(VerifiedDeadDrop.isMeaningfulSignature(signature: nullSignature))

        let emptySignature: HexEncodedString? = HexEncodedString(bytes: [])
        XCTAssertFalse(VerifiedDeadDrop.isMeaningfulSignature(signature: emptySignature))

        let allZeroSignature: HexEncodedString = HexEncodedString(bytes: [0x00, 0x00, 0x00])
        XCTAssertFalse(VerifiedDeadDrop.isMeaningfulSignature(signature: allZeroSignature))

        let meaningfulSignature: HexEncodedString = HexEncodedString(bytes: [0x01, 0x02, 0x03])
        XCTAssertTrue(VerifiedDeadDrop.isMeaningfulSignature(signature: meaningfulSignature))
    }
}
