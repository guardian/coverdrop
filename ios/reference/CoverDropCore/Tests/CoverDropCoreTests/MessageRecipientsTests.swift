@testable import CoverDropCore
import XCTest

final class MessageRecipientsTests: XCTestCase {
    private let context = IntegrationTestScenarioContext(scenario: .multipleJournalists)

    func testDefaultRecipient() throws {
        let keys = try context.loadKeysVerified(step: "001_initial_state")
        let expectedDefaultRecipientId = keys.defaultJournalistId
        XCTAssert(expectedDefaultRecipientId == nil)
    }

    func testDesks() throws {
        let keys = try context.loadKeysVerified(step: "001_initial_state")
        let messageRecipients = try MessageRecipients(
            verifiedPublicKeys: keys,
            excludingDefaultRecipient: false
        )
        XCTAssertEqual(messageRecipients.desks.count, 0)
    }

    func testJournalists() throws {
        let keys = try context.loadKeysVerified(step: "001_initial_state")
        let messageRecipients = try MessageRecipients(
            verifiedPublicKeys: keys,
            excludingDefaultRecipient: false
        )
        XCTAssertEqual(messageRecipients.journalists.count, 2)
    }
}
