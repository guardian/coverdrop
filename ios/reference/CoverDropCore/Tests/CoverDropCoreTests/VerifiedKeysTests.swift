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
            "b046107cd0c8dbacaa9d848d70057e119c51c93634e7b6872f905e1b0e22cf5d"
        )
    }

    func testGetJounalistMessagingKeys() async throws {
        // this scenario and step combination offers multiple journalist msg keys
        let context = IntegrationTestScenarioContext(scenario: .messaging)
        let verifiedPublicKeys = try context.loadKeysVerified(step: "003_journalist_replied_and_processed")

        let journalistMessageKeys = verifiedPublicKeys
            .allMessageKeysForJournalistId(journalistId: "static_test_journalist")
        let mostRecentJournalistMessageKey = journalistMessageKeys.max(by: { $0.notValidAfter < $1.notValidAfter })
        XCTAssertEqual(
            mostRecentJournalistMessageKey?.key.key.hexStr,
            "bd481b297b520da85730de0e3cfc76e375c26583388c12f209f32ecfb73f4715"
        )
    }
}
