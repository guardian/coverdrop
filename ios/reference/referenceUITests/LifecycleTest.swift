import Foundation
import XCTest

final class LifecycleUITest: XCTestCase {
    var app: XCUIApplication!
    var timeout: TimeInterval = .init(20)

    override func setUpWithError() throws {
        try super.setUpWithError()
        // is usually best to stop immediately when a failure occurs.
        continueAfterFailure = false
    }

    override func tearDownWithError() throws {
        captureScreenshotOnFailure()
    }

    func testappOnline_CoverDropDisabled_ShouldShowErrorPageTest() {
        let app = XCUIApplicationLauncher.launch(testingFlags: [
            .coverDropDisabled
        ])

        _ = app.buttons["Open CoverDrop"].waitForExistence(timeout: timeout)
        app.buttons["Open CoverDrop"].tap()
        XCTAssertTrue(app.staticTexts["Secure messaging unavailable"].exists)
    }

    func testappOffline_CoverDropDisabled_ShouldShowErrorPageTest() {
        let app = XCUIApplicationLauncher.launch(testingFlags: [
            .coverDropDisabled, .offline
        ])

        _ = app.buttons["Open CoverDrop"].waitForExistence(timeout: timeout)
        app.buttons["Open CoverDrop"].tap()
        XCTAssertTrue(app.staticTexts["Secure messaging unavailable"].exists)
    }

    func testappOffline_CoverDropEnabled_ShouldFailWithoutCache() {
        let app = XCUIApplicationLauncher.launch(testingFlags: [
            .offline
        ])

        _ = app.buttons["Open CoverDrop"].waitForExistence(timeout: timeout)
        app.buttons["Open CoverDrop"].tap()
        XCTAssertTrue(app.staticTexts["error: verifiedPublicKeysNotAvailable"].exists)
    }

    func testappOffline_CoverDropEnabled_ShouldSucceedWithCache() {
        let app = XCUIApplicationLauncher.launch(testingFlags: [
            .startWithCachedPublicKeys, .offline
        ])

        _ = app.buttons["Open CoverDrop"].waitForExistence(timeout: timeout)
        app.buttons["Open CoverDrop"].tap()
        _ = app.buttons["Dismiss and ignore warnings"].waitForExistence(timeout: timeout)
        app.buttons["Dismiss and ignore warnings"].tap()
        _ = app.buttons["Get started"].waitForExistence(timeout: timeout)
        XCTAssertTrue(app.buttons["Get started"].isEnabled)
    }

    func testappOnline_CoverDropEnabledAfterInit_ShouldSucceedOnForeground() {
        let app = XCUIApplicationLauncher.launch(testingFlags: [
            .coverDropDisabled
        ])

        _ = app.buttons["Open CoverDrop"].waitForExistence(timeout: timeout)
        app.buttons["Open CoverDrop"].tap()
        XCTAssertTrue(app.staticTexts["Secure messaging unavailable"].exists)

        // Simulate remote config change after app is launched
        let devMenuButton = app.buttons["toggleDevMenuButton"]
        devMenuButton.tap()

        let enableCoverDropButton = app.switches["toggleCoverDropEnabledButton"].switches.firstMatch
        enableCoverDropButton.tap()

        let closeDevMenuButton = app.buttons["closeDevMenuButton"]
        closeDevMenuButton.tap()

        // Background the app
        XCUIDevice.shared.press(XCUIDevice.Button.home)

        // Foreground the app to trigger the reinitialisation of coverdrop service
        app.activate()

        _ = app.buttons["Dismiss and ignore warnings"].waitForExistence(timeout: timeout)
        app.buttons["Dismiss and ignore warnings"].tap()
        _ = app.buttons["Get started"].waitForExistence(timeout: timeout)
        XCTAssertTrue(app.buttons["Get started"].isEnabled)

        // Simulate remote config change after app is launched
        let devMenuButton2 = app.buttons["toggleDevMenuButton"]
        devMenuButton2.tap()

        let enableCoverDropButton2 = app.switches["toggleCoverDropEnabledButton"].switches.firstMatch
        enableCoverDropButton2.tap()

        XCUIDevice.shared.press(XCUIDevice.Button.home)

        // Foreground the app to trigger the reinitialisation of coverdrop service
        app.activate()

        XCTAssertTrue(app.staticTexts["Secure messaging unavailable"].exists)
    }
}
