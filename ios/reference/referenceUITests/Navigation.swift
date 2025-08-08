import CoverDropCore
import XCTest

enum Navigation {
    public class NavigationState {
        public init(password: [String]? = nil) {
            self.password = password
        }

        var password: [String]?
    }

    static func navigateToOnboarding(in app: XCUIApplication, with timeout: TimeInterval = 20) {
        _ = app.buttons["Open CoverDrop"].waitForExistence(timeout: timeout)
        app.buttons["Open CoverDrop"].tap()
        _ = app.buttons["Dismiss and ignore warnings"].waitForExistence(timeout: timeout)
        app.buttons["Dismiss and ignore warnings"].tap()
        _ = app.buttons["Get started"].waitForExistence(timeout: timeout)
        app.buttons["Get started"].wait(until: { $0.isEnabled })
        app.buttons["Get started"].tap()
        app.buttons["Yes, start conversation"].tap()
    }

    static func navigateToPassphraseFromStartPage(in app: XCUIApplication,
                                                  with _: TimeInterval = 20) -> NavigationState {
        app.buttons["Get started"].wait(until: { $0.isEnabled })
        app.buttons["Get started"].tap()
        app.buttons["Yes, start conversation"].tap()

        app.buttons["Continue"].tap()
        app.buttons["Continue"].tap()
        app.buttons["Set up my passphrase"].tap()
        let predicate = NSPredicate(format: "label CONTAINS[c] 'Reveal passphrase to continue'")
        app.buttons.containing(predicate).element.tap()

        let password = Array(1 ... StaticConfig.devConfig.passphraseWordCount)
            .map {
                app.staticTexts.matching(identifier: "Word \($0)").allElementsBoundByIndex.first!.label
            }
        let state = NavigationState(password: password)
        return state
    }

    static func navigateToPassphraseAfterOnboarding(in app: XCUIApplication,
                                                    with _: TimeInterval = 20) -> NavigationState {
        app.buttons["Continue"].tap()
        app.buttons["Continue"].tap()
        app.buttons["Set up my passphrase"].tap()
        let predicate = NSPredicate(format: "label CONTAINS[c] 'Reveal passphrase to continue'")
        app.buttons.containing(predicate).element.tap()

        let password = Array(1 ... StaticConfig.devConfig.passphraseWordCount)
            .map {
                app.staticTexts.matching(identifier: "Word \($0)").allElementsBoundByIndex.first!.label
            }
        let state = NavigationState(password: password)
        return state
    }

    static func startEnteringPassphrase(in app: XCUIApplication, state: NavigationState,
                                        with timeout: TimeInterval = 20) -> NavigationState {
        app.buttons["I have remembered my passphrase"].tap()

        // Awaiting async function
        _ = app.textFields["Passphrase Word 1"].waitForExistence(timeout: timeout)
        _ = app.textFields["Passphrase Word 2"].waitForExistence(timeout: timeout)
        _ = app.textFields["Passphrase Word 3"].waitForExistence(timeout: timeout)

        for (index, label) in state.password!.enumerated() {
            let field = app.textFields["Passphrase Word \(index + 1)"]
            field.tap()
            field.typeText(label)
        }
        return state
    }

    static func navigateToNewMessage(in app: XCUIApplication) {
        app.buttons["Confirm passphrase"].tap()

        // awaiting async function
        _ = app.staticTexts["What do you want to share with us?"].waitForExistence(timeout: 20)
    }

    static func selectTestJournalist(in app: XCUIApplication) {
        app.buttons["Change recipient"].tap()
        app.buttons["Journalists"].tap()
        let journalistSelected = "Static Test Journalist"
        app.buttons["Select \(journalistSelected)"].tap()
        XCTAssert(app.staticTexts["Selected Recipient is \(journalistSelected)"].exists)
    }

    static func composeAndSendMessage(message: String, in app: XCUIApplication, with _: TimeInterval = 20) {
        let messageField = app.textViews["Compose your message"]
        messageField.tap()
        messageField.typeText(message)
        app.buttons["Send message"].tap()
    }

    static func assertForcedSingleRecpient(in app: XCUIApplication) {
        app.staticTexts["Selected Recipient is Static Test Journalist"].tap()
        XCTAssert(app.staticTexts["At the current time you can only contact a single Guardian recipient."].exists)
        app.buttons["Dismiss"].tap()
    }

    static func loginToInbox(in app: XCUIApplication, state: NavigationState, with timeout: TimeInterval = 20) {
        _ = app.buttons["Check your message vault"].waitForExistence(timeout: timeout)
        app.buttons["Check your message vault"].tap()

        _ = app.textFields["Passphrase Word 1"].waitForExistence(timeout: timeout)
        _ = app.textFields["Passphrase Word 2"].waitForExistence(timeout: timeout)
        _ = app.textFields["Passphrase Word 3"].waitForExistence(timeout: timeout)

        for (index, label) in state.password!.enumerated() {
            let field = app.textFields["Passphrase Word \(index + 1)"]
            XCTAssertTrue(field.isHittable)
            if !field.hasFocus {
                // Tap the text field to give it keyboard focus
                field.tap()
            }
            field.typeText(label)
        }

        app.buttons["Confirm passphrase"].tap()
    }
}
