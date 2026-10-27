@testable import CoverDropCore
import Sodium
import XCTest

final class VerifiedKeysTests: XCTestCase {
    func testGetCoverNodeKeys() throws {
        let context = IntegrationTestScenarioContext(scenario: .messaging)
        let verifiedPublicKeys = try context.loadKeysVerified(step: "004_dead_drop_expired_and_no_longer_displayed")

        let verifiedCoverNodeMessagingKeys = verifiedPublicKeys.mostRecentCoverNodeMessagingKeysFromAllHierarchies()

        let publicKeysData = try context.loadKeys(step: "004_dead_drop_expired_and_no_longer_displayed")
        let expectedKey = try XCTUnwrap(publicKeysData.keys
            .flatMap { $0.coverNodes }
            .flatMap { $0.covernodes["covernode_001"] ?? [] }
            .flatMap { $0.msgPks }
            .max { $0.notValidAfter.date < $1.notValidAfter.date })
        XCTAssertEqual(verifiedCoverNodeMessagingKeys["covernode_001"]?.key.key, expectedKey.key.bytes)
    }

    func testGetJournalistMessagingKeys() throws {
        // this scenario and step combination offers multiple journalist msg keys
        let context = IntegrationTestScenarioContext(scenario: .messaging)
        let verifiedPublicKeys = try context.loadKeysVerified(step: "003_journalist_replied_and_processed")

        let verifiedJournalistMessageKeys = verifiedPublicKeys
            .allMessageKeysForJournalistId(journalistId: "static_test_journalist")

        let publicKeysData = try context.loadKeys(step: "003_journalist_replied_and_processed")
        let expectedKey = try XCTUnwrap(publicKeysData.keys
            .flatMap { $0.journalists }
            .flatMap { $0.journalists["static_test_journalist"] ?? [] }
            .flatMap { $0.msgPks }
            .max { $0.notValidAfter.date < $1.notValidAfter.date })

        let mostRecentJournalistMessageKey = verifiedJournalistMessageKeys
            .max(by: { $0.notValidAfter < $1.notValidAfter })
        XCTAssertEqual(mostRecentJournalistMessageKey?.key.key, expectedKey.key.bytes)
    }
}
