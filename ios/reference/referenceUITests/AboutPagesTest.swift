import Foundation
import XCTest

final class AboutPageUITest: XCTestCase {
    let timeout = TimeInterval(20)

    override func setUpWithError() throws {
        try super.setUpWithError()

        // In UI tests it is usually best to stop immediately when a failure occurs.
        continueAfterFailure = false
    }

    override func tearDownWithError() throws {
        captureScreenshotOnFailure()
    }

    func testAboutAndThenPrivacyPageNavigation() {
        let app = XCUIApplicationLauncher.launch()

        app.buttons["Open CoverDrop"].tap()
        _ = app.buttons["Dismiss and ignore warnings"].waitForExistence(timeout: timeout)
        app.buttons["Dismiss and ignore warnings"].tap()
        app.buttons["About Secure Messaging"].tap()
        XCTAssert(app.staticTexts["About Secure Messaging"].exists)
        app.buttons["Privacy policy"].tap()
        XCTAssert(app.staticTexts["Secure Messaging privacy policy"].exists)
    }
}
