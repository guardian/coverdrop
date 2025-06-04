@testable import CoverDropCore
import XCTest

final class SecretDataTests: XCTestCase {
    func testGetJournalistKeyDataForJournalistId() async throws {
        let context = IntegrationTestScenarioContext(scenario: .minimal)
        let verifiedPublicKeys = try context.loadKeysVerified()

        let journalistId = "static_test_journalist"
        let data = verifiedPublicKeys.getJournalistKeyDataForJournalistId(journalistId: journalistId)

        XCTAssertEqual(data?.recipientId, journalistId)
    }
}
