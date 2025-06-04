@testable import CoverDropCore
import Sodium
import XCTest

final class VerifiedKeysTests: XCTestCase {
    func testGetCoverNodeKeys() async throws {
        let context = IntegrationTestScenarioContext(scenario: .messaging)
        let verifiedPublicKeys = try context.loadKeysVerified(step: "004_dead_drop_expired_and_no_longer_displayed")

        let coverNodeMessagingKeys = verifiedPublicKeys.mostRecentCoverNodeMessagingKeysFromAllHierarchies()
        XCTAssertEqual(
            coverNodeMessagingKeys["covernode_001"]?.key.key.hexStr,
            "13c50b83ad24030b2b2d5a49d7438abe6609988b399f2345e0362be39504c45a"
        )
    }

    func testGetJounalistMessagingKeys() async throws {
        // this scenario and step combination offers multiple journalist msg keys
        let context = IntegrationTestScenarioContext(scenario: .messaging)
        let verifiedPublicKeys = try context.loadKeysVerified(step: "004_dead_drop_expired_and_no_longer_displayed")

        let journalistMessageKeys = verifiedPublicKeys
            .allMessageKeysForJournalistId(journalistId: "static_test_journalist")
        let mostRecentJournalistMessageKey = journalistMessageKeys.max(by: { $0.notValidAfter < $1.notValidAfter })
        XCTAssertEqual(
            mostRecentJournalistMessageKey?.key.key.hexStr,
            "ab780e21bca74478152c75f0d5071a5a6fcbbdbb18b6c6addb206e707b7e2e5a"
        )
    }
}
