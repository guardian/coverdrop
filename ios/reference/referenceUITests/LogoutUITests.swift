import Foundation
import XCTest

final class LogoutUITest: XCTestCase {
    let timeout = TimeInterval(20)

    override func setUpWithError() throws {
        try super.setUpWithError()

        // In UI tests it is usually best to stop immediately when a failure occurs.
        continueAfterFailure = false
    }

    override func tearDownWithError() throws {
        captureScreenshotOnFailure()
    }

    func testLogoutAfterMessagesSendAndLoginAgainShowsInbox() throws {
        let app = XCUIApplicationLauncher.launch()

        Navigation.navigateToOnboarding(in: app, with: timeout)
        let state = Navigation.navigateToPassphraseAfterOnboarding(in: app, with: timeout)
        _ = Navigation.startEnteringPassphrase(in: app, state: state)
        Navigation.navigateToNewMessage(in: app)
        Navigation.selectTestJournalist(in: app)
        Navigation.composeAndSendMessage(message: "Hey!", in: app)

        _ = app.staticTexts["Your message will be received by a journalist soon."].waitForExistence(timeout: timeout)

        XCTAssertTrue(app.staticTexts["Your message will be received by a journalist soon."].exists)

        app.buttons["Log out of vault"].tap()
        _ = app.buttons["Log out"].waitForExistence(timeout: timeout)
        app.buttons["Log out"].tap()

        Navigation.loginToInbox(in: app, state: state)
        _ = app.staticTexts["Messaging with"].waitForExistence(timeout: timeout)
        XCTAssertTrue(app.staticTexts["Messaging with"].exists)
    }
}
