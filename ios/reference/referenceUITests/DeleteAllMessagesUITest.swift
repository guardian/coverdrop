import Foundation
import XCTest

final class DeletedAllMessagesUITest: XCTestCase {
    let timeout = TimeInterval(20)

    override func setUpWithError() throws {
        try super.setUpWithError()

        // In UI tests it is usually best to stop immediately when a failure occurs.
        continueAfterFailure = false
    }

    override func tearDownWithError() throws {
        captureScreenshotOnFailure()
    }

    // checks all memory state is removed after delete messages action is
    // taken and a new session is created
    func testDeleteMessagesInInbox() throws {
        let app = XCUIApplicationLauncher.launch(testingFlags: [.startWithNonEmptyStorage])
        let state = navigateToInboxWithExistingStorage(app: app)

        let deleteButtonExists = app.buttons["Delete message vault"].waitForExistence(timeout: timeout)
        XCTAssertTrue(deleteButtonExists)

        app.buttons["Delete message vault"].tap()

        let deleteDialogExists = app.buttons["Delete everything"].waitForExistence(timeout: timeout)
        XCTAssertTrue(deleteDialogExists)

        app.buttons["Delete everything"].tap()

        Navigation.loginToInbox(in: app, state: state)

        let predicate = NSPredicate(format: "label CONTAINS[c] 'Failed to open message vault'")
        let errorMessage = XCUIApplication().staticTexts.containing(predicate).element

        _ = errorMessage.waitForExistence(timeout: timeout)
        XCTAssertTrue(errorMessage.exists)

        app.buttons["I do not have a passphrase yet"].tap()
        let continueExists = app.buttons["Continue"].waitForExistence(timeout: timeout)

        XCTAssertTrue(continueExists)

        let newState = Navigation.navigateToPassphraseAfterOnboarding(in: app)
        assertRecipientIsDefault(app: app, state: newState)
    }

    // This tests the leave inbox button from the inbox page clears items in memory
    // and logs the user out
    func testLeaveInboxDeletesMemoryInInboxTest() throws {
        let app = XCUIApplicationLauncher.launch(testingFlags: [.startWithNonEmptyStorage])
        _ = navigateToInboxWithExistingStorage(app: app)

        app.buttons["Leave vault"].tap()
        _ = app.buttons["Log out"].waitForExistence(timeout: timeout)
        app.buttons["Log out"].tap()

        let state = Navigation.navigateToPassphraseFromStartPage(in: app)
        assertRecipientIsDefault(app: app, state: state)
    }

    func navigateToInboxWithExistingStorage(app: XCUIApplication) -> Navigation.NavigationState {
        let passphrase = ["external", "jersey", "squeeze"]
        let state = Navigation.NavigationState(password: passphrase)

        let openCoverdropExists = app.buttons["Open CoverDrop"].waitForExistence(timeout: timeout)
        XCTAssertTrue(openCoverdropExists)

        app.buttons["Open CoverDrop"].tap()

        let dismissWarningButtonExists = app.buttons["Dismiss and ignore warnings"].waitForExistence(timeout: timeout)
        XCTAssertTrue(dismissWarningButtonExists)
        app.buttons["Dismiss and ignore warnings"].tap()

        Navigation.loginToInbox(in: app, state: state)
        _ = app.staticTexts["Messaging with"].waitForExistence(timeout: timeout)
        return state
    }

    func assertRecipientIsDefault(app: XCUIApplication, state: Navigation.NavigationState) {
        _ = Navigation.startEnteringPassphrase(in: app, state: state)
        Navigation.navigateToNewMessage(in: app)
        _ = app.staticTexts["No recipient selected"].waitForExistence(timeout: timeout)
        XCTAssertTrue(app.staticTexts["No recipient selected"].exists)
    }
}
