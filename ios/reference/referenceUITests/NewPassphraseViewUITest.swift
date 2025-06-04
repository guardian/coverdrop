import Foundation
import XCTest

final class NewPassphraseUITest: XCTestCase {
    var app: XCUIApplication!
    var timeout: TimeInterval = .init(20)

    override func setUpWithError() throws {
        try super.setUpWithError()
        app = XCUIApplicationLauncher.launch()
        // is usually best to stop immediately when a failure occurs.
        continueAfterFailure = false
    }

    override func tearDownWithError() throws {
        captureScreenshotOnFailure()
    }

    /// Given   the user visits the remember passphrase screen
    /// When   the user clicks show passphrase button
    /// Then    they are shown the passphrase text

    func testShowPassphraseButtonThenTheyAreShownThePassphraseText() {
        Navigation.navigateToOnboarding(in: app, with: timeout)
        app.buttons["Continue"].tap()
        app.buttons["Continue"].tap()
        app.buttons["Set up my passphrase"].tap()
        let predicate = NSPredicate(format: "label CONTAINS[c] 'Reveal passphrase to continue'")
        _ = app.buttons.containing(predicate).element.waitForExistence(timeout: timeout)
        app.buttons.containing(predicate).element.tap()
        XCTAssert(app.buttons["I have remembered my passphrase"].isHittable)
    }

    /// Given    the user visits the remember passphrase screen
    /// When    it is the first interaction
    /// Then     the passphrase is hidden

    func testPassphraseIsHidden() {
        Navigation.navigateToOnboarding(in: app, with: timeout)
        app.buttons["Continue"].tap()
        app.buttons["Continue"].tap()
        app.buttons["Set up my passphrase"].tap()
        _ = app.staticTexts["••••••"].waitForExistence(timeout: timeout)
        XCTAssert(app.staticTexts["••••••"].exists)
    }

    /// Given   the user visits the remember passphrase screen
    /// When   the user clicks the back button
    /// Then    they are taken back to the Onboarding page
    func testBackNavigation() {
        Navigation.navigateToOnboarding(in: app, with: timeout)
        app.buttons["Continue"].tap()
        app.buttons["Continue"].tap()
        app.buttons["Set up my passphrase"].tap()
        app.buttons["Go Back"].tap()
        XCTAssert(app.staticTexts["How this works"].exists)
    }

    /// Given    the user visits the remember passphrase screen and the passphrase is shown
    /// When    the user clicks the hide password button
    /// Then    the passphrase is hidden

    func testHidePassphraseButton() {
        Navigation.navigateToOnboarding(in: app, with: timeout)
        app.buttons["Continue"].tap()
        app.buttons["Continue"].tap()
        app.buttons["Set up my passphrase"].tap()
        let predicate = NSPredicate(format: "label CONTAINS[c] 'Reveal passphrase fields'")
        _ = app.buttons.containing(predicate).element.waitForExistence(timeout: timeout)
        app.buttons.containing(predicate).element.tap()
        let hidePredicate = NSPredicate(format: "label CONTAINS[c] 'Hide passphrase fields'")
        _ = app.buttons.containing(hidePredicate).element.waitForExistence(timeout: timeout)
        app.buttons.containing(hidePredicate).element.tap()
        XCTAssert(app.staticTexts["••••••"].exists)
    }
}
