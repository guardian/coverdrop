import Foundation
import XCTest

final class SecurityAlertTest: XCTestCase {
    let timeout = TimeInterval(20)

    override func setUpWithError() throws {
        try super.setUpWithError()

        // In UI tests it is usually best to stop immediately when a failure occurs.
        continueAfterFailure = false
    }

    override func tearDownWithError() throws {
        captureScreenshotOnFailure()
    }

    func testSecurityAlertsShowCorrectErrors() {
        let app = XCUIApplicationLauncher.launch()

        _ = app.buttons["Open CoverDrop"].waitForExistence(timeout: timeout)
        app.buttons["Open CoverDrop"].tap()
        XCTAssertTrue(app.staticTexts["- An active debugger is attached to this process"].exists)
        XCTAssertTrue(app.staticTexts["- This is running in an emulator"].exists)
        XCTAssertTrue(app.staticTexts["- This device has evidence of reverse engineering"].exists)
        _ = app.buttons["Dismiss and ignore warnings"].waitForExistence(timeout: timeout)
        app.buttons["Dismiss and ignore warnings"].tap()
    }
}
