import XCTest

final class SelectRecipientUITests: XCTestCase {
    var app: XCUIApplication!
    let timeout = TimeInterval(20)

    override func setUpWithError() throws {
        try super.setUpWithError()
        app = XCUIApplication()
        XCUIApplicationLauncher.launch(with: app)
        // In UI tests it is usually best to stop immediately when a failure occurs.
        continueAfterFailure = false
    }

    /// AC-RS-1
    /// Given    user visits the recipient selection screen
    /// When    loaded
    /// Then    all desks are shown

    func testDesksShown() {
        Navigation.navigateToOnboarding(in: app, with: timeout)
        let state = Navigation.navigateToPassphraseAfterOnboarding(in: app, with: timeout)
        Navigation.startEnteringPassphrase(in: app, state: state)
        Navigation.navigateToNewMessage(in: app)
        Navigation.navigateToSelectRecipient(in: app)

        XCTAssert(app.buttons["Generated Test Desk"].isHittable)
    }

    /// AC-RS-2
    /// Given   user visits the recipient selection screen
    /// When    journalist tab is clicked
    /// Then    all journalists are shown

    func testJournalistsShow() {
        Navigation.navigateToOnboarding(in: app, with: timeout)
        let state = Navigation.navigateToPassphraseAfterOnboarding(in: app, with: timeout)
        Navigation.startEnteringPassphrase(in: app, state: state)
        Navigation.navigateToNewMessage(in: app)
        Navigation.navigateToSelectRecipient(in: app)

        app.buttons["Journalists"].tap()

        XCTAssert(app.staticTexts["Generated Test Journalist"].exists)
    }

    /// AC-RS-3
    /// Given   user visits the recipient selection screen
    /// When    a desk item is clicked
    /// Then    the desk confirmation screen is shown

    func testDeskDetailScreenShown() {
        Navigation.navigateToOnboarding(in: app, with: timeout)
        let state = Navigation.navigateToPassphraseAfterOnboarding(in: app, with: timeout)
        Navigation.startEnteringPassphrase(in: app, state: state)
        Navigation.navigateToNewMessage(in: app)
        Navigation.navigateToSelectRecipient(in: app)

        app.buttons["Generated Test Desk"].tap()

        let pageTitle = app.staticTexts["Generated Test Desk"]
        XCTAssert(pageTitle.exists)

        let deskDetail = app.staticTexts["This is a test desk"]
        XCTAssert(deskDetail.exists)
    }

    /// AC-RS-4
    /// Given   user visits the recipient selection screen
    /// When    a desk item is clicked and then "Select desk" is clicked
    /// Then    the desk is returned as confirmed id

    func testDeskSelected() {
        Navigation.navigateToOnboarding(in: app, with: timeout)
        let state = Navigation.navigateToPassphraseAfterOnboarding(in: app, with: timeout)
        Navigation.startEnteringPassphrase(in: app, state: state)
        Navigation.navigateToNewMessage(in: app)
        Navigation.navigateToSelectRecipient(in: app)

        let deskSelected = "Generated Test Desk"

        app.buttons[deskSelected].tap()
        app.buttons["Select team"].tap()

        XCTAssert(app.staticTexts["Selected Recipient is \(deskSelected)"].exists)
    }

    /// AC-RS-5
    /// Given   user visits the recipient selection screen
    /// When    the "Select" button next to a journalist is clicked
    /// Then    the journalist is returned as confirmed id

    func testJournalistSelected() {
        Navigation.navigateToOnboarding(in: app, with: timeout)
        let state = Navigation.navigateToPassphraseAfterOnboarding(in: app, with: timeout)
        Navigation.startEnteringPassphrase(in: app, state: state)
        Navigation.navigateToNewMessage(in: app)
        Navigation.navigateToSelectRecipient(in: app)

        app.buttons["Journalists"].tap()

        let journalistSelected = "Generated Test Journalist"

        app.buttons["Select \(journalistSelected)"].tap()

        XCTAssert(app.staticTexts["Selected Recipient is \(journalistSelected)"].exists)
    }
}
