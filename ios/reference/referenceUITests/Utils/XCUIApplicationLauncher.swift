@testable import CoverDropCore
import Foundation
import XCTest

enum XCUIApplicationLauncher {
    static func launch(testingFlags: [TestingFlag] = []) -> XCUIApplication {
        let app = XCUIApplication()

        // we always want to disable animations
        var allFlags = testingFlags
        allFlags.append(TestingFlag.disableAnimations)

        // update the launch arguments with all flags
        TestingBridge.setTestingFlags(launchArguments: &app.launchArguments, flags: allFlags)

        app.launch()

        return app
    }
}
