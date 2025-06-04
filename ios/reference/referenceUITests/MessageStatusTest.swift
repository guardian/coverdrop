import Foundation
import XCTest

final class MessageStatusUITest: XCTestCase {
    let timeout = TimeInterval(20)

    override func setUpWithError() throws {
        try super.setUpWithError()

        // In UI tests it is usually best to stop immediately when a failure occurs.
        continueAfterFailure = false
    }

    override func tearDownWithError() throws {
        captureScreenshotOnFailure()
    }

    func skipped_testNewMessagesHavePendingStateThenSendAfterAppBackground() throws {
        let app = XCUIApplicationLauncher.launch()

        Navigation.navigateToOnboarding(in: app, with: timeout)
        let state = Navigation.navigateToPassphraseAfterOnboarding(in: app, with: timeout)
        _ = Navigation.startEnteringPassphrase(in: app, state: state)
        Navigation.navigateToNewMessage(in: app)
        Navigation.selectTestJournalist(in: app)
        Navigation.composeAndSendMessage(message: "Hey!", in: app)

        _ = app.staticTexts["Your message will be received by a journalist soon."].waitForExistence(timeout: timeout)

        XCTAssertTrue(app.staticTexts["Your message will be received by a journalist soon."].exists)

        app.buttons["Review conversation"].tap()

        _ = app.staticTexts["Messaging with"].waitForExistence(timeout: timeout)
        app.staticTexts["Messaging with"].tap()

        _ = app.staticTexts["Pending"].waitForExistence(timeout: timeout)

        XCTAssertTrue(app.staticTexts["Pending"].exists)

        // The coverDrop app sends messages on app foreground,
        // so that we can seperate user interaction (ie composing a message) with the actual
        // network request being made. This is why we need to background and foreground the app to
        // get the message sent and have the status change.
        XCUIDevice.shared.press(XCUIDevice.Button.home)

        // wait until we are outside the minimum time between background sends
        sleep(11)
        // Foreground the app
        app.activate()

        _ = app.staticTexts["Sent"].waitForExistence(timeout: timeout)
        XCTAssertTrue(app.staticTexts["Sent"].exists)
    }

    func skipped_testNewMessagesHavePendingStateThenSendAfterAppRestart() throws {
        var app = XCUIApplicationLauncher.launch()

        Navigation.navigateToOnboarding(in: app, with: timeout)
        let state = Navigation.navigateToPassphraseAfterOnboarding(in: app, with: timeout)
        _ = Navigation.startEnteringPassphrase(in: app, state: state)
        Navigation.navigateToNewMessage(in: app)
        Navigation.selectTestJournalist(in: app)
        Navigation.composeAndSendMessage(message: "Hey!", in: app)

        _ = app.staticTexts["Your message will be received by a journalist soon."].waitForExistence(timeout: timeout)

        XCTAssertTrue(app.staticTexts["Your message will be received by a journalist soon."].exists)

        app.buttons["Review conversation"].tap()

        _ = app.staticTexts["Messaging with"].waitForExistence(timeout: timeout)
        app.staticTexts["Messaging with"].tap()

        _ = app.staticTexts["Pending"].waitForExistence(timeout: timeout)

        XCTAssertTrue(app.staticTexts["Pending"].exists)

        // The coverDrop app sends messages on app foreground,
        // so that we can seperate user interaction (ie composing a message) with the actual
        // network request being made. This is why we need to background and foreground the app to
        // get the message sent and have the status change.
        app.terminate()
        // wait until we are outside the minimum time between background sends
        sleep(22)

        app = XCUIApplicationLauncher.launch()
        _ = app.buttons["Open CoverDrop"].waitForExistence(timeout: timeout)
        app.buttons["Open CoverDrop"].tap()

        _ = app.buttons["Dismiss and ignore warnings"].waitForExistence(timeout: timeout)
        app.buttons["Dismiss and ignore warnings"].tap()

        Navigation.loginToInbox(in: app, state: state)

        _ = app.staticTexts["Messaging with"].waitForExistence(timeout: timeout)
        app.staticTexts["Messaging with"].tap()

        _ = app.staticTexts["Sent"].waitForExistence(timeout: timeout)
        XCTAssertTrue(app.staticTexts["Sent"].exists)
    }
}
