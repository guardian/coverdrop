import Foundation
import XCTest

final class NavigationUITest: XCTestCase {
    var app: XCUIApplication!

    let timeout = TimeInterval(20)

    override func setUpWithError() throws {
        try super.setUpWithError()
        // In UI tests it is usually best to stop immediately when a failure occurs.
        continueAfterFailure = false
    }

    override func tearDownWithError() throws {
        captureScreenshotOnFailure()
    }

    //    Given I'm a logged out user
    //    When I press the Get started button
    //    And create a new session
    //    Then I see the new conversation window

    //    Given I'm a logged in user
    //    When I send my first message from the new conversation screen
    //    Then I see the journalist chat view

    //    Given I'm a logged in user
    //    When I send my first message from the new conversation screen
    //    And then view the conversation from the inbox
    //    And I navigate back
    //    Then I should be on the inbox view

    func testNewConversationFlowWithBackNavigation() throws {
        let app = XCUIApplicationLauncher.launch()

        Navigation.navigateToOnboarding(in: app, with: timeout)
        let state = Navigation.navigateToPassphraseAfterOnboarding(in: app, with: timeout)
        _ = Navigation.startEnteringPassphrase(in: app, state: state)
        Navigation.navigateToNewMessage(in: app)
        Navigation.selectTestJournalist(in: app)
        Navigation.composeAndSendMessage(message: "Hey!", in: app)

        _ = app.staticTexts["Your message will be received by a journalist soon."].waitForExistence(timeout: timeout)

        XCTAssertTrue(app.staticTexts["Your message will be received by a journalist soon."].exists)

        app.buttons["Review conversation"].tap()

        _ = app.staticTexts["Messaging with"].waitForExistence(timeout: timeout)
        app.staticTexts["Messaging with"].tap()

        _ = app.staticTexts["Pending"].waitForExistence(timeout: timeout)

        _ = app.buttons["Go Back"].waitForExistence(timeout: timeout)
        app.buttons["Go Back"].tap()
        _ = app.staticTexts["Messaging with"].waitForExistence(timeout: timeout)
        XCTAssertTrue(app.staticTexts["Messaging with"].exists)
    }

    // GIVEN there is only one journalist available
    // WHEN I navigate to the new conversation screen
    // THEN the journalist should be pre-selected and cannot be changed
    func testNewConversationWithForcedSingleRecipient() {
        let app = XCUIApplicationLauncher.launch(testingFlags: [.forceSingleRecipient])

        Navigation.navigateToOnboarding(in: app, with: timeout)
        let state = Navigation.navigateToPassphraseAfterOnboarding(in: app, with: timeout)
        _ = Navigation.startEnteringPassphrase(in: app, state: state)
        Navigation.navigateToNewMessage(in: app)
        Navigation.assertForcedSingleRecpient(in: app)
    }

    //    Given I'm a logged out user
    //    When I abandon the new conversation screen before sending a message
    //    And log in again
    //    Then I should see the new conversation screen with a warning message

    //    Given I'm a logged out user
    //    When I abandon the new conversation screen before sending a message
    //    And log in again
    //    Then I should see the new conversation screen with a warning message

    func skipping_testNewConversationAbandonFlowWithNewLogin() throws {
        let app = XCUIApplicationLauncher.launch()

        Navigation.navigateToOnboarding(in: app, with: timeout)
        let state = Navigation.navigateToPassphraseAfterOnboarding(in: app, with: timeout)
        _ = Navigation.startEnteringPassphrase(in: app, state: state)
        Navigation.navigateToNewMessage(in: app)

        _ = app.staticTexts["What do you want to share with us?"].waitForExistence(timeout: timeout)

        let pageTitle = app.staticTexts["What do you want to share with us?"]

        XCTAssertTrue(pageTitle.exists)

        _ = app.buttons["Log out of vault"].waitForExistence(timeout: timeout)
        app.buttons["Log out of vault"].tap()
        _ = app.buttons["Log out"].waitForExistence(timeout: timeout)
        app.buttons["Log out"].tap()
        app.buttons["Get started"].wait(until: { $0.isEnabled })
        XCTAssertTrue(app.buttons["Get started"].exists)

        Navigation.loginToInbox(in: app, state: state)
        let messageText = app.staticTexts["Enter your message to start a conversation."]
        messageText.wait(until: { $0.isEnabled })
        XCTAssertTrue(messageText.exists)
    }

    //    Given I'm a logged out user
    //    When I abandon the new conversation screen before sending a message
    //    And create a new session
    //    Then I should not see my previous recipient

    func testNewConversationAbandonFlowWithNewSession() throws {
        let app = XCUIApplicationLauncher.launch()

        Navigation.navigateToOnboarding(in: app, with: timeout)
        let state = Navigation.navigateToPassphraseAfterOnboarding(in: app, with: timeout)
        _ = Navigation.startEnteringPassphrase(in: app, state: state)
        Navigation.navigateToNewMessage(in: app)

        _ = app.staticTexts["What do you want to share with us?"].waitForExistence(timeout: timeout)

        let pageTitle = app.staticTexts["What do you want to share with us?"]

        XCTAssertTrue(pageTitle.exists)

        _ = app.buttons["Log out of vault"].waitForExistence(timeout: timeout)
        app.buttons["Log out of vault"].tap()
        _ = app.buttons["Log out"].waitForExistence(timeout: timeout)
        app.buttons["Log out"].tap()

        let newState = Navigation.navigateToPassphraseFromStartPage(in: app)
        _ = Navigation.startEnteringPassphrase(in: app, state: newState)
        Navigation.navigateToNewMessage(in: app)
        _ = app.staticTexts["No recipient selected"].waitForExistence(timeout: timeout)
        XCTAssertTrue(app.staticTexts["No recipient selected"].exists)
    }

    //    Given I’m a logged out user who has previously send a message
    //    When I log in
    //    Then I should see the inbox view

    //
    //    Given I am a logged in user who has previously sent a message
    //    When I’m on the inbox view and choose a conversation
    //    Then I should see the conversation view
    func testLogingWithExistingMessageCanViewCoversationFlow() throws {
        let app = XCUIApplicationLauncher.launch()

        Navigation.navigateToOnboarding(in: app, with: timeout)
        let state = Navigation.navigateToPassphraseAfterOnboarding(in: app, with: timeout)
        _ = Navigation.startEnteringPassphrase(in: app, state: state)
        Navigation.navigateToNewMessage(in: app)
        Navigation.selectTestJournalist(in: app)
        Navigation.composeAndSendMessage(message: "Hey!", in: app)
        app.buttons["Log out of vault"].tap()
        _ = app.buttons["Log out"].waitForExistence(timeout: timeout)
        app.buttons["Log out"].tap()
        Navigation.loginToInbox(in: app, state: state)
        _ = app.staticTexts["Messaging with"].waitForExistence(timeout: timeout)
        XCTAssertTrue(app.staticTexts["Messaging with"].exists)
        app.staticTexts["Messaging with"].tap()
        XCTAssertTrue(app.staticTexts["Hey!"].exists)
    }

    func testNewConversationWithoutRecipientError() throws {
        let app = XCUIApplicationLauncher.launch(testingFlags: [.mockedDataNoDefaultJournalist])

        Navigation.navigateToOnboarding(in: app, with: timeout)
        let state = Navigation.navigateToPassphraseAfterOnboarding(in: app, with: timeout)
        _ = Navigation.startEnteringPassphrase(in: app, state: state)
        Navigation.navigateToNewMessage(in: app)
        // with no message the send message should be disabled
        XCTAssertFalse(app.buttons["Send message"].isEnabled)
        // We add a message here to show that the send message button is also disabled
        // if you have a message but no recipient
        let messageField = app.textViews["Compose your message"]
        messageField.tap()
        messageField.typeText("hey")
        XCTAssertFalse(app.buttons["Send message"].isEnabled)
    }
}
