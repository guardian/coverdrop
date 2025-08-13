import Foundation
import XCTest

final class KeyExpiryUITest: XCTestCase {
    var app: XCUIApplication!

    let timeout = TimeInterval(20)

    override func setUpWithError() throws {
        try super.setUpWithError()

        // In UI tests it is usually best to stop immediately when a failure occurs.
        continueAfterFailure = false
    }

    override func tearDownWithError() throws {
        captureScreenshotOnFailure()
    }

    func testAppWithMissingKeysCanOpenExistingConversation() {
        let app = XCUIApplicationLauncher.launch(testingFlags: [
            .mockedDataExpiredMessagesScenario,
            .startWithNonEmptyStorage
        ])

        let passphrase = ["external", "jersey", "squeeze"]
        let state = Navigation.NavigationState(password: passphrase)

        _ = app.buttons["Open CoverDrop"].waitForExistence(timeout: timeout)
        app.buttons["Open CoverDrop"].tap()
        _ = app.buttons["Dismiss and ignore warnings"].waitForExistence(timeout: timeout)
        app.buttons["Dismiss and ignore warnings"].tap()

        // Update system time from menu
        let devMenuButton = app.buttons["toggleDevMenuButton"]
        _ = devMenuButton.waitForExistence(timeout: timeout)
        devMenuButton.tap()

        let advanceTimeButton = app.buttons["advanceTimeButton"]
        _ = advanceTimeButton.waitForExistence(timeout: timeout)
        advanceTimeButton.tap()

        let closeDevMenuButton = app.buttons["closeDevMenuButton"]
        closeDevMenuButton.tap()

        Navigation.loginToInbox(in: app, state: state)

        _ = app.staticTexts["Static Test Journalist"].waitForExistence(timeout: timeout)
        app.staticTexts["Static Test Journalist"].tap()
        XCTAssertTrue(app.staticTexts["Hey this is pending"].exists)
        _ = app.staticTexts["Static Test Journalist is currently unavailable."].waitForExistence(timeout: timeout)
        XCTAssertTrue(
            app
                .staticTexts["Static Test Journalist is currently unavailable."].exists
        )
    }
}
