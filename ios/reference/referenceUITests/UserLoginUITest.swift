import Foundation
import XCTest

final class UserLoginUITest: XCTestCase {
    var app: XCUIApplication!

    let timeout = TimeInterval(20)

    override func setUpWithError() throws {
        try super.setUpWithError()
        app = XCUIApplication()
        app = XCUIApplication()
        XCUIApplicationLauncher.launch(with: app)
        // In UI tests it is usually best to stop immediately when a failure occurs.
        continueAfterFailure = false
    }

    /// AC-EP-1
    /// Given   the user visits the enter passphrase screen
    /// When   they type a word into the first text field
    /// Then    the passphrase is hidden

    /// AC-EP-2
    /// Given    the user visits the enter passphrase screen and they typed a word into the first text field
    /// When    the user clicks unhide passphrase icon
    /// Then    they are shown the passphrase text
    func testPassphraseFieldsAreHiddenOnFirstView() {
        Navigation.navigateToOnboarding(in: app, with: timeout)
        let state = Navigation.navigateToPassphraseAfterOnboarding(in: app, with: timeout)

        app.buttons["I have remembered my passphrase"].tap()

        // Awaiting async function
        _ = app.secureTextFields["Passphrase Word 1"].waitForExistence(timeout: timeout)

        for index in state.password!.indices {
            let field = app.secureTextFields["Passphrase Word \(index + 1)"]
            XCTAssertTrue(field.isHittable)
        }

        for index in state.password!.indices {
            app.buttons["show \(index + 1)"].tap()
            let field = app.textFields["Passphrase Word \(index + 1)"]
            field.tap()
            field.typeText("mooop")
            XCTAssertTrue(field.isHittable)
        }
    }

    /// AC-EP-3
    /// Given     the user visits the enter passphrase screen
    /// When    the user has not entered all valid words into the text fields
    /// Then    an error message is shown

    func testSubmitPassphraseDisabledOnInvalidEntry() {
        Navigation.navigateToOnboarding(in: app, with: timeout)
        let state = Navigation.navigateToPassphraseAfterOnboarding(in: app, with: timeout)

        app.buttons["I have remembered my passphrase"].tap()

        // Awaiting async function
        _ = app.secureTextFields["Passphrase Word 1"].waitForExistence(timeout: timeout)

        // swiftlint:disable:next unused_enumerated
        for (index, _) in state.password!.enumerated() {
            let field = app.secureTextFields["Passphrase Word \(index + 1)"]
            field.tap()
            field.typeText("mooop")
        }

        app.buttons["Confirm passphrase"].tap()

        // Awaiting async function
        _ = app.staticTexts["The passphrase cannot be valid because it contains words that are not on the word list."]
            .waitForExistence(timeout: timeout)

        let errorTitle = app
            .staticTexts["The passphrase cannot be valid because it contains words that are not on the word list."]

        XCTAssertTrue(errorTitle.exists)
    }

    /// AC-EP-4
    /// Given     the user visits the enter passphrase screen
    /// When    the user has entered all valid words into the text fields
    /// Then    the confirm passphrase button is enabled

    /// AC-EP-5
    /// Given     the user visits the enter passphrase screen and the user has entered all valid words into the text
    /// fields
    /// When    the confirm passphrase button is pressed
    /// Then    the secure storage should be unlocked and the user should be taken to the new message page

    func testLoginWithCorrectPassphraseText() {
        Navigation.navigateToOnboarding(in: app, with: timeout)
        let state = Navigation.navigateToPassphraseAfterOnboarding(in: app, with: timeout)
        _ = Navigation.startEnteringPassphrase(in: app, state: state)

        XCTAssertTrue(app.buttons["Confirm passphrase"].isEnabled)

        app.buttons["Confirm passphrase"].tap()

        // Awaiting async function
        _ = app.staticTexts["What do you want to share with us?"].waitForExistence(timeout: timeout)

        let pageTitle = app.staticTexts["What do you want to share with us?"]

        XCTAssertTrue(pageTitle.exists)
    }

    /// AC-EP-6
    /// Given    the user visits the enter passphrase screen and the user has entered all valid words into the text
    /// fields but have not been issued this passphrase previously
    /// When    the confirm passphrase button is pressed
    /// Then     the secure storage should remain locked and an error message should be displayed; the error message
    /// should mention that this could mean that no storage was previously created

    func testSubmitNewCorrectPassphraseMakesNewStorage() {
        Navigation.navigateToOnboarding(in: app, with: timeout)
        let state = Navigation.navigateToPassphraseAfterOnboarding(in: app, with: timeout)

        app.buttons["I have remembered my passphrase"].tap()

        // Awaiting async function
        _ = app.secureTextFields["Passphrase Word 1"].waitForExistence(timeout: timeout)

        // swiftlint:disable:next unused_enumerated
        for (index, _) in state.password!.enumerated() {
            let field = app.secureTextFields["Passphrase Word \(index + 1)"]
            field.tap()
            field.typeText("musket")
        }

        app.buttons["Confirm passphrase"].tap()

        // Awaiting async function
        _ = app
            .staticTexts["Either a wrong password was provided or Secure Messaging has never been used on this device."]
            .waitForExistence(timeout: timeout)

        let errorTitle = app
            .staticTexts["The passphrase you entered does not match the generated one from the previous screen."]

        XCTAssertTrue(errorTitle.exists)
    }

    /// AC-EP-7
    /// Given     the user visits the enter passphrase screen
    /// When    the back button is pressed
    /// Then    the user is taken back to the choose conversation screen

    func testBackButton() {
        Navigation.navigateToOnboarding(in: app, with: timeout)
        let labels = Navigation.navigateToPassphraseAfterOnboarding(in: app, with: timeout)

        app.buttons["I have remembered my passphrase"].tap()

        // Awaiting async function
        _ = app.secureTextFields["Passphrase Word 1"].waitForExistence(timeout: timeout)

        app.buttons["Close login"].tap()

        XCTAssert(app.buttons["Get started"].isHittable)
    }
}
