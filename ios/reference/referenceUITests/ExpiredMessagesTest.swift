import CoverDropCore
import Foundation
import XCTest

final class ExpiredMessagesTest: XCTestCase {
    var app: XCUIApplication!

    let timeout = TimeInterval(20)

    override func setUpWithError() throws {
        try super.setUpWithError()

        app = XCUIApplicationLauncher.launch(testingFlags: [
            .mockedDataExpiredMessagesScenario,
            .startWithNonEmptyStorage
        ])

        // In UI tests it is usually best to stop immediately when a failure occurs.
        continueAfterFailure = false
    }

    override func tearDownWithError() throws {
        captureScreenshotOnFailure()
    }

    func skipping_testExpiredMessagesAreShown() {
        Navigation.navigateToOnboarding(in: app, with: timeout)
        let state = Navigation.navigateToPassphraseAfterOnboarding(in: app, with: timeout)
        _ = Navigation.startEnteringPassphrase(in: app, state: state)
        Navigation.navigateToNewMessage(in: app)

        app.buttons["Change recipient"].tap()
        app.buttons["Journalists"].tap()

        let journalistSelected = "Generated Test Journalist"

        app.buttons["Select \(journalistSelected)"].tap()

        XCTAssert(app.staticTexts["Selected Recipient is \(journalistSelected)"].exists)

        let messageTextEditor = app.textViews["Compose your message"]

        messageTextEditor.tap()
        messageTextEditor.typeText("Hey bro")

        app.buttons["Send message"].tap()

        app.buttons["Review conversation"].tap()

        _ = app.staticTexts["Messaging with"].waitForExistence(timeout: timeout)
        app.staticTexts["Messaging with"].tap()

        let predicate = NSPredicate(format: "label CONTAINS[c] 'Expiring in'")
        let expiresMessage = XCUIApplication().staticTexts.containing(predicate).element

        _ = expiresMessage.waitForExistence(timeout: 20)

        XCTAssertTrue(expiresMessage.exists)
    }
}
