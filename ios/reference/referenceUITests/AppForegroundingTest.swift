import Foundation
import XCTest

final class AppForegroundUITest: XCTestCase {
    let timeout = TimeInterval(20)

    override func setUpWithError() throws {
        try super.setUpWithError()

        // In UI tests it is usually best to stop immediately when a failure occurs.
        continueAfterFailure = false
    }

    override func tearDownWithError() throws {
        captureScreenshotOnFailure()
    }

    func testForegroundingAppFailsOnIncorrectPublicKeysDataAndDeadDrops() {
        let app = XCUIApplicationLauncher.launch(testingFlags: [
            .mockedDataEmptyKeysData
        ])

        _ = app.buttons["Open CoverDrop"].waitForExistence(timeout: timeout)
        app.buttons["Open CoverDrop"].tap()
        XCTAssertTrue(app.staticTexts["error: failedToLoadPublicKeys"].exists)
    }

    func testForegroundingAppStartsOnCorrectPublicKeysDataAndDeadDrops() {
        let app = XCUIApplicationLauncher.launch()

        _ = app.buttons["Open CoverDrop"].waitForExistence(timeout: timeout)
        app.buttons["Open CoverDrop"].tap()
        _ = app.buttons["Dismiss and ignore warnings"].waitForExistence(timeout: timeout)
        app.buttons["Dismiss and ignore warnings"].tap()
        _ = app.buttons["Get started"].waitForExistence(timeout: timeout)
        XCTAssertTrue(app.buttons["Get started"].isEnabled)
    }
}
