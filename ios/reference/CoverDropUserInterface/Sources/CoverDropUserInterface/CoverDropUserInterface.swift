import CoverDropCore
import Foundation
import SwiftUI

public enum CoverDropUserInterface {
    public static func application(
        _: UIApplication,
        shouldAllowExtensionPointIdentifier extensionPointIdentifier: UIApplication.ExtensionPointIdentifier
    ) -> Bool {
        return extensionPointIdentifier != .keyboard
    }

    public static func applicationDidEnterBackground(_ window: UIWindow?, coverProvider: AppSwitchingCoverProvider) {
        // Covers the app's content in the app switcher.
        window?.addSubview(coverProvider.coverView)
    }

    public static func applicationWillEnterForeground(coverProvider: AppSwitchingCoverProvider) {
        // Removes the privacy cover view from the app.
        coverProvider.coverView.removeFromSuperview()
    }

    public static func initialView(
        config: CoverDropConfig,
        uiConfig: CoverDropUserInterfaceConfiguration
    ) -> AnyView {
        return AnyView(ContentView(config: config, uiConfig: uiConfig))
    }
}
