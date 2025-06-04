import Foundation

@Observable
public class CoverDropUserInterfaceConfiguration {
    public init(
        showAboutScreenDebugInformation: Bool,
        showBetaBanner: Bool
    ) {
        self.showAboutScreenDebugInformation = showAboutScreenDebugInformation
        self.showBetaBanner = showBetaBanner
    }

    /// Whether to show debug information at the bottom of the about screen. These include
    /// general diagnostics about the app state and recent background operations. It does not
    /// include any sensitive information.
    public var showAboutScreenDebugInformation: Bool

    /// Whether to show the beta banner at the top of the app that highlights the app is in beta.
    public var showBetaBanner: Bool
}
