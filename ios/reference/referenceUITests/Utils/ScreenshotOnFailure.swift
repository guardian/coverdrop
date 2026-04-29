import XCTest

extension XCTestCase {
    func captureScreenshotOnFailure() {
        if testRun?.hasSucceeded == false {
            let app = XCUIApplication()
            let screenshot = app.screenshot()
            let attachment = XCTAttachment(screenshot: screenshot)
            attachment.name = "Screenshot on Failure"
            attachment.lifetime = .deleteOnSuccess // Only keep for failed tests
            add(attachment)
        }
    }
}
