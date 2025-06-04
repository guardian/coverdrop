import CoverDropCore
import Foundation
import XCTest

final class SendMessageUITest: XCTestCase {
    var app: XCUIApplication!

    let timeout = TimeInterval(20)

    override func setUpWithError() throws {
        try super.setUpWithError()
        app = XCUIApplication()
        XCUIApplicationLauncher.launch(with: app)
        // In UI tests it is usually best to stop immediately when a failure occurs.
        continueAfterFailure = false
    }

    func testMessageWithCorrectPassphraseText() {
        let app = XCUIApplication()
        Navigation.navigateToOnboarding(in: app, with: timeout)
        let state = Navigation.navigateToPassphraseAfterOnboarding(in: app, with: timeout)
        Navigation.startEnteringPassphrase(in: app, state: state)
        Navigation.navigateToNewMessage(in: app)
        Navigation.navigateToSelectRecipient(in: app)
        Navigation.sendMessageToTestJournalist(message: "Hey", in: app)

        _ = app.staticTexts["Your message will be received by a journalist soon."].waitForExistence(timeout: timeout)

        XCTAssertTrue(app.staticTexts["Your message will be received by a journalist soon."].exists)

        app.buttons["Go to your inbox"].tap()

        app.staticTexts["Messaging with"].waitForExistence(timeout: timeout)
        app.staticTexts["Messaging with"].tap()
    }

    // User should be logged out if they press the back button from the sent message screen

    func testMessageWithCorrectPassphraseTextThenLogout() {
        let app = XCUIApplication()
        Navigation.navigateToOnboarding(in: app, with: timeout)
        let state = Navigation.navigateToPassphraseAfterOnboarding(in: app, with: timeout)
        Navigation.startEnteringPassphrase(in: app, state: state)
        Navigation.navigateToNewMessage(in: app)
        Navigation.navigateToSelectRecipient(in: app)
        Navigation.sendMessageToTestJournalist(message: "Hey", in: app)

        _ = app.staticTexts["Your message will be received by a journalist soon."].waitForExistence(timeout: timeout)

        XCTAssertTrue(app.staticTexts["Your message will be received by a journalist soon."].exists)

        app.buttons["Go Back"].waitForExistence(timeout: timeout)
        app.buttons["Go Back"].tap()

        app.buttons["Get started"].waitForExistence(timeout: timeout)
        XCTAssertTrue(app.buttons["Get started"].isEnabled)
    }

    func testMessageLengthWithDifferentMessageSizes() {
        Navigation.navigateToOnboarding(in: app, with: timeout)
        let state = Navigation.navigateToPassphraseAfterOnboarding(in: app, with: timeout)
        Navigation.startEnteringPassphrase(in: app, state: state)
        Navigation.navigateToNewMessage(in: app)
        Navigation.navigateToSelectRecipient(in: app)
        Navigation.sendMessageToTestJournalist(message: "Hey !", in: app)

        _ = app.staticTexts["Your message will be received by a journalist soon."].waitForExistence(timeout: timeout)

        XCTAssertTrue(app.staticTexts["Your message will be received by a journalist soon."].exists)

        app.buttons["Go to your inbox"].tap()

        app.staticTexts["Messaging with"].waitForExistence(timeout: timeout)
        app.staticTexts["Messaging with"].tap()

        app.buttons["Send a new message"].waitForExistence(timeout: timeout)
        app.buttons["Send a new message"].tap()

        let message = "Hey"

        let messageField = app.textViews["Compose your message"]
        messageField.tap()
        messageField.typeText(message)

        XCTAssertFalse(app.staticTexts["Message limit reached"].exists)

        messageField.doubleTap()

        app.menuItems["Cut"].tap()

        let longMessage = """
        Lorem ipsum dolor sit amet, consectetur adipiscing elit. Integer dolor
            nulla, ornare et tristique imperdiet, dictum sit amet velit. Curabitur pharetra erat sed
            neque interdum, non mattis tortor auctor. Curabitur eu ipsum ac neque semper eleifend.
            Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus.
            Integer erat mi, ultrices nec arcu ut, sagittis sollicitudin est. In hac habitasse
            platea dictumst. Sed in efficitur elit. Curabitur nec commodo elit. Aliquam tincidunt
            rutrum nisl ut facilisis. Aenean ornare ut mauris eget lacinia. Mauris a felis quis orci
            auctor varius sit amet eget est. Curabitur a urna sit amet diam sagittis aliquet eget eu
            sapien. Curabitur a pharetra purus.
            Nulla facilisi. Suspendisse potenti. Morbi mollis aliquet sapien sed faucibus. Donec
            aliquam nibh nibh, ac faucibus felis aliquam at. Pellentesque egestas enim sem, eu
            tempor urna posuere eget. Cras fermentum commodo neque ac gravida.
        """
        messageField.tap()
        messageField.typeText(longMessage)

        XCTAssertTrue(app.staticTexts["Message limit reached"].exists)
    }
}
