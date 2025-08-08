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
        deadDropData.deadDrops[1].signature.bytes[0] = deadDropData.deadDrops[1].signature.bytes[0] ^ 0x01

        let result = VerifiedDeadDrops.fromAllDeadDropData(deadDrops: deadDropData, verifiedKeys: verifiedKeys)
        XCTAssertEqual(result.deadDrops.map { $0.id }, [1, 3])
    }
}
