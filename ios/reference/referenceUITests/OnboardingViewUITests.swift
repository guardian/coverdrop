import XCTest

final class OnboardingViewUITests: XCTestCase {
    var app: XCUIApplication!
    var timeout: TimeInterval = .init(20)

    override func setUpWithError() throws {
        try super.setUpWithError()
        app = XCUIApplication()
        XCUIApplicationLauncher.launch(with: app)
        // In UI tests it is usually best to stop immediately when a failure occurs.
        continueAfterFailure = false
    }

    func testNavigateToOnboarding() {
        Navigation.navigateToOnboarding(in: app, with: timeout)

        let pageTitle = app.staticTexts["How this works"]
        XCTAssert(pageTitle.exists)
    }

    /// Given   the user visits the how this works screen
    /// When    the user clicks "Continue"
    /// Then    they are shown the remember passphrase screen

    func testContinueButton() {
        Navigation.navigateToOnboarding(in: app, with: timeout)

        app.buttons["Continue"].tap()

        let rememberPasssphraseScreen = app.staticTexts["Remember Passphrase"]

        XCTAssert(rememberPasssphraseScreen.exists)
    }

    /// Given   the user visits the how this works screen
    /// When    the user clicks the back button
    /// Then    they are taken back to the start screen

    func testNavigateBack() {
        Navigation.navigateToOnboarding(in: app, with: timeout)

        app.buttons["Close onboarding"].tap()

        let startScreen = app.staticTexts["Send us a message securely and privately"]

        XCTAssert(startScreen.exists)
    }
}
