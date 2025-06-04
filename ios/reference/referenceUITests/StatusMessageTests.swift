import Foundation
import XCTest

final class StatusMessageTests: XCTestCase {
    var app: XCUIApplication!
    var timeout: TimeInterval = .init(20)

    override func setUpWithError() throws {
        try super.setUpWithError()
        app = XCUIApplication()

        // is usually best to stop immediately when a failure occurs.
        continueAfterFailure = false
    }

    func testStatusMessageNotShownOnAvailableStatus() {
        XCUIApplicationLauncher.launch(with: app)
        _ = app.buttons["Open CoverDrop"].waitForExistence(timeout: timeout)
        app.buttons["Open CoverDrop"].tap()
        _ = app.buttons["Dismiss and ignore warnings"].waitForExistence(timeout: timeout)
        app.buttons["Dismiss and ignore warnings"].tap()
        app.buttons["Get started"].wait(until: { $0.isEnabled })
        XCTAssert(app.buttons["Get started"].isHittable)
    }

    func testStatusMessageShownOnUnavailableStatus() {
        app.launchArguments += ["STATUS_UNAVAILABLE"]
        XCUIApplicationLauncher.launch(with: app)
        _ = app.buttons["Open CoverDrop"].waitForExistence(timeout: timeout)
        app.buttons["Open CoverDrop"].tap()
        _ = app.buttons["Dismiss and ignore warnings"].waitForExistence(timeout: timeout)
        app.buttons["Dismiss and ignore warnings"].tap()
        let predicate =
            NSPredicate(format: "label CONTAINS[c] 'The Secure Messaging feature is currently not available'")
        let messageText = XCUIApplication().staticTexts.containing(predicate).element

        messageText.waitForExistence(timeout: timeout)
        XCTAssert(messageText.exists)
    }
}
