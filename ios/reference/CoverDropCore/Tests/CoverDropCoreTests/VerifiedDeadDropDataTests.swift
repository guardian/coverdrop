@testable import CoverDropCore
import XCTest

final class VerifiedDeadDropDataTests: XCTestCase {
    /// Verified dead drops are identified by published date; the API-assigned id is not signed.
    private func publishedSeconds(_ verified: VerifiedDeadDrops) -> [Int64] {
        return verified.deadDrops.map { Int64($0.publishedDate.timeIntervalSince1970) }
    }

    func testVerification_happyPath() throws {
        let testContext = IntegrationTestScenarioContext(scenario: .minimal)
        let verifiedKeys = try testContext.loadKeysVerified()
        let deadDropData = try testContext.loadDeadDrop()
        let result = VerifiedDeadDrops.fromAllDeadDropData(deadDrops: deadDropData, verifiedKeys: verifiedKeys)
        XCTAssertEqual(publishedSeconds(result), deadDropData.deadDrops.map { $0.createdAt.epochSeconds })
    }

    func testVerification_whenDateManipulated_thenSkipped() throws {
        let testContext = IntegrationTestScenarioContext(scenario: .minimal)
        let verifiedKeys = try testContext.loadKeysVerified()
        var deadDropData = try testContext.loadDeadDrop()

        let expected = [deadDropData.deadDrops[0], deadDropData.deadDrops[2]].map { $0.createdAt.epochSeconds }

        // manipulate the date
        deadDropData.deadDrops[1].createdAt =
            try RFC3339DateTimeString(date: deadDropData.deadDrops[1].createdAt.date.minusSeconds(1))

        let result = VerifiedDeadDrops.fromAllDeadDropData(deadDrops: deadDropData, verifiedKeys: verifiedKeys)
        XCTAssertEqual(publishedSeconds(result), expected)
    }

    func testVerification_whenDataManipulated_thenSkipped() throws {
        let testContext = IntegrationTestScenarioContext(scenario: .minimal)
        let verifiedKeys = try testContext.loadKeysVerified()
        var deadDropData = try testContext.loadDeadDrop()

        let expected = [deadDropData.deadDrops[0], deadDropData.deadDrops[2]].map { $0.createdAt.epochSeconds }

        // manipulate the data
        deadDropData.deadDrops[1].data.bytes[0] = deadDropData.deadDrops[1].data.bytes[0] ^ 0x01

        let result = VerifiedDeadDrops.fromAllDeadDropData(deadDrops: deadDropData, verifiedKeys: verifiedKeys)
        XCTAssertEqual(publishedSeconds(result), expected)
    }

    func testVerification_whenSignatureManipulated_thenSkipped() throws {
        let testContext = IntegrationTestScenarioContext(scenario: .minimal)
        let verifiedKeys = try testContext.loadKeysVerified()
        var deadDropData = try testContext.loadDeadDrop()

        let expected = [deadDropData.deadDrops[0], deadDropData.deadDrops[2]].map { $0.createdAt.epochSeconds }

        // manipulate the signature
        deadDropData.deadDrops[1].signature.bytes[0] = deadDropData.deadDrops[1].signature.bytes[0] ^ 0x01

        let result = VerifiedDeadDrops.fromAllDeadDropData(deadDrops: deadDropData, verifiedKeys: verifiedKeys)
        XCTAssertEqual(publishedSeconds(result), expected)
    }

    /// The API assigns `id` after signing, so verification must not depend on it.
    func testVerification_whenIdManipulated_thenVerifiedResultIsUnchanged() throws {
        let testContext = IntegrationTestScenarioContext(scenario: .minimal)
        let verifiedKeys = try testContext.loadKeysVerified()
        var deadDropData = try testContext.loadDeadDrop()

        let expected = deadDropData.deadDrops.map { $0.createdAt.epochSeconds }

        // manipulate the id
        deadDropData.deadDrops[1].id = Int.max

        let result = VerifiedDeadDrops.fromAllDeadDropData(deadDrops: deadDropData, verifiedKeys: verifiedKeys)
        XCTAssertEqual(publishedSeconds(result), expected)
    }

    /// Varying the unsigned sub-second component still verifies, so the timestamp must be truncated.
    func testVerification_whenSubSecondPrecisionManipulated_thenTimestampIsTruncatedToSignedPrecision() throws {
        let testContext = IntegrationTestScenarioContext(scenario: .minimal)
        let verifiedKeys = try testContext.loadKeysVerified()
        var deadDropData = try testContext.loadDeadDrop()

        let original = VerifiedDeadDrops.fromAllDeadDropData(deadDrops: deadDropData, verifiedKeys: verifiedKeys)

        // vary the sub-second component, leaving the signed whole-second value untouched
        let signedSeconds = deadDropData.deadDrops[1].createdAt.epochSeconds
        deadDropData.deadDrops[1].createdAt = RFC3339DateTimeString(
            date: Date(timeIntervalSince1970: TimeInterval(signedSeconds) + 0.999)
        )

        let result = VerifiedDeadDrops.fromAllDeadDropData(deadDrops: deadDropData, verifiedKeys: verifiedKeys)

        XCTAssertEqual(result.deadDrops.count, original.deadDrops.count)
        XCTAssertEqual(result.deadDrops.map { $0.publishedDate }, original.deadDrops.map { $0.publishedDate })
    }
}
