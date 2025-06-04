import CoverDropCore
import CoverDropUserInterface
import Foundation
import GuardianFonts
import SwiftUI
import UIKit

@main
class AppDelegate: NSObject, UIApplicationDelegate {
    var window: UIWindow?
    var backgroundView: UIView?
    let coverProvider = CoverdropCoverProvider()

    public let coverDropService = CoverDropService.shared

    public var config: CoverDropConfig {
        var config: StaticConfig = .prodConfig

        #if DEBUG
            config = .devConfig
        #endif

        #if CODE
            config = .codeConfig
        #endif

        #if STAGING
            config = .stagingConfig
        #endif
        return config
    }

    func application(
        _: UIApplication,
        didFinishLaunchingWithOptions _: [UIApplication.LaunchOptionsKey: Any]?
    ) -> Bool {
        let window = UIWindow(frame: UIScreen.main.bounds)
        self.window = window
        window.rootViewController = UIHostingController(rootView: AppInitalView())
        window.makeKeyAndVisible()
        GuardianFonts.registerFonts()

        disableAnimationsIfNeeded()
        try? coverDropService.didLaunch(config: config)

        return true
    }

    /// Disables UIView animations for the purposes of UI Testiing
    private func disableAnimationsIfNeeded() {
        if TestingBridge.isEnabled(.disableAnimations) {
            UIView.setAnimationsEnabled(false)
        }
    }

    func applicationDidEnterBackground(_: UIApplication) {
        CoverDropService.didEnterBackground()
        CoverDropUserInterface.applicationDidEnterBackground(window, coverProvider: coverProvider)
    }

    func applicationWillEnterForeground(_: UIApplication) {
        CoverDropService.willEnterForeground(config: config)
        CoverDropUserInterface.applicationWillEnterForeground(coverProvider: coverProvider)
    }

    func application(
        _ application: UIApplication,
        shouldAllowExtensionPointIdentifier extensionPointIdentifier: UIApplication.ExtensionPointIdentifier
    ) -> Bool {
        return CoverDropUserInterface.application(
            application,
            shouldAllowExtensionPointIdentifier: extensionPointIdentifier
        )
    }
}
