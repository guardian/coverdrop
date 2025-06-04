import Foundation
import XCTest

final class MessageStatusUITest: XCTestCase {
    var app: XCUIApplication!

    let timeout = TimeInterval(20)

    override func setUpWithError() throws {
        try super.setUpWithError()

        // In UI tests it is usually best to stop immediately when a failure occurs.
        continueAfterFailure = false
    }

    func testNewMessagesHavePendingStateThenSendState() throws {
        app = XCUIApplication()

        XCUIApplicationLauncher.launch(with: app)

        Navigation.navigateToOnboarding(in: app, with: timeout)
        let state = Navigation.navigateToPassphraseAfterOnboarding(in: app, with: timeout)
        Navigation.startEnteringPassphrase(in: app, state: state)
        Navigation.navigateToNewMessage(in: app)
        Navigation.navigateToSelectRecipient(in: app)
        Navigation.sendMessageToTestJournalist(message: "Hey!", in: app)

        _ = app.staticTexts["Your message will be received by a journalist soon."].waitForExistence(timeout: timeout)

        XCTAssertTrue(app.staticTexts["Your message will be received by a journalist soon."].exists)

        app.buttons["Go to your inbox"].tap()

        app.staticTexts["Messaging with"].waitForExistence(timeout: timeout)
        app.staticTexts["Messaging with"].tap()

        _ = app.staticTexts["Pending"].waitForExistence(timeout: timeout)

        XCTAssertTrue(app.staticTexts["Pending"].exists)

        // The coverDrop app only actually sends messages on app foregrounding
        // so that we can seperate user interaction (ie composing a message) with the actual
        // network request being made. This is why we need to background and foreground the app to
        // get the message sent and have the status change.

        XCUIDevice.shared.press(XCUIDevice.Button.home)

        app.activate()

        _ = app.staticTexts["Sent"].waitForExistence(timeout: timeout)

        XCTAssertTrue(app.staticTexts["Sent"].exists)
    }

    func testNewMessagesHavePendingStateThenSendAfterAppRestart() throws {
        app = XCUIApplication()

        XCUIApplicationLauncher.launch(with: app)

        Navigation.navigateToOnboarding(in: app, with: timeout)
        let state = Navigation.navigateToPassphraseAfterOnboarding(in: app, with: timeout)
        Navigation.startEnteringPassphrase(in: app, state: state)
        Navigation.navigateToNewMessage(in: app)
        Navigation.navigateToSelectRecipient(in: app)
        Navigation.sendMessageToTestJournalist(message: "Hey!", in: app)

        _ = app.staticTexts["Your message will be received by a journalist soon."].waitForExistence(timeout: timeout)

        XCTAssertTrue(app.staticTexts["Your message will be received by a journalist soon."].exists)

        app.buttons["Go to your inbox"].tap()

        app.staticTexts["Messaging with"].waitForExistence(timeout: timeout)
        app.staticTexts["Messaging with"].tap()

        _ = app.staticTexts["Pending"].waitForExistence(timeout: timeout)

        XCTAssertTrue(app.staticTexts["Pending"].exists)

        // The coverDrop app only actually sends messages on app foregrounding
        // so that we can seperate user interaction (ie composing a message) with the actual
        // network request being made. This is why we need to background and foreground the app to
        // get the message sent and have the status change.

        XCUIApplicationLauncher.launch(with: app)
        _ = app.buttons["Open CoverDrop"].waitForExistence(timeout: timeout)
        app.buttons["Open CoverDrop"].tap()

        _ = app.buttons["Dismiss and ignore warnings"].waitForExistence(timeout: timeout)
        app.buttons["Dismiss and ignore warnings"].tap()

        Navigation.loginToInbox(in: app, state: state)

        app.staticTexts["Messaging with"].waitForExistence(timeout: timeout)
        app.staticTexts["Messaging with"].tap()

        _ = app.staticTexts["Sent"].waitForExistence(timeout: timeout)
        XCTAssertTrue(app.staticTexts["Sent"].exists)
    }
}
