import Foundation
import XCTest

final class InboxUITest: XCTestCase {
    let timeout = TimeInterval(20)

    override func setUpWithError() throws {
        try super.setUpWithError()

        // In UI tests it is usually best to stop immediately when a failure occurs.
        continueAfterFailure = false
    }

    override func tearDownWithError() throws {
        captureScreenshotOnFailure()
    }

    func skipped_testConversationCannotBeRepliedToIfInactive() throws {
        // This START_WITH_MESSAGES launch argument is required to add test messages to the inbox.
        // This is needed because we only decrypt messages for journalists already in your inbox,
        // so without these inital messages we cannot process the dead drop fixtures
        let app = XCUIApplicationLauncher.launch(testingFlags: [
            .startWithNonEmptyStorage,
            .mockedDataMultipleJournalists
        ])

        let passphrase = ["external", "jersey", "squeeze"]
        let state = Navigation.NavigationState(password: passphrase)

        _ = app.buttons["Open CoverDrop"].waitForExistence(timeout: timeout)
        app.buttons["Open CoverDrop"].tap()
        _ = app.buttons["Dismiss and ignore warnings"].waitForExistence(timeout: timeout)
        app.buttons["Dismiss and ignore warnings"].tap()

        Navigation.loginToInbox(in: app, state: state)

        _ = app.staticTexts["Static Test Journalist"].waitForExistence(timeout: timeout)
        app.staticTexts["Static Test Journalist"].tap()
        app.staticTexts.matching(identifier: "This conversation has been closed")
    }

    func testConversationCanBeRepliedToIfJournalistIsLastSender() throws {
        // This START_WITH_MESSAGES launch argument is required to add test messages to the inbox.
        // This is needed because we only decrypt messages for journalists already in your inbox,
        // so without these inital messages we cannot process the dead drop fixtures
        let app = XCUIApplicationLauncher.launch(testingFlags: [
            .startWithNonEmptyStorage
        ])

        let passphrase = ["external", "jersey", "squeeze"]
        let state = Navigation.NavigationState(password: passphrase)

        _ = app.buttons["Open CoverDrop"].waitForExistence(timeout: timeout)
        app.buttons["Open CoverDrop"].tap()
        _ = app.buttons["Dismiss and ignore warnings"].waitForExistence(timeout: timeout)
        app.buttons["Dismiss and ignore warnings"].tap()

        Navigation.loginToInbox(in: app, state: state)

        _ = app.staticTexts["Static Test Journalist"].waitForExistence(timeout: timeout)
        app.staticTexts["Static Test Journalist"].tap()
        _ = app.textViews["Compose your message"].waitForExistence(timeout: timeout)
        XCTAssertTrue(app.textViews["Compose your message"].exists)
    }
}
