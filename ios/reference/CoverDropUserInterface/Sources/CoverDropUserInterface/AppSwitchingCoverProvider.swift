import UIKit

/// This protocol provides the view required for covering the host app's screen when the app moves into the background.
/// The view provided can be customized to suit the branding of the host app. This security feature will ensure that the
/// app content is obscured whether the user is using Coverdrop or not.
public protocol AppSwitchingCoverProvider {
    /// Provides the frame of the app's screen bounds to the `coverView` to ensure that the view extends to the bounds
    /// of the screen. The default implementation returns `UIScreen.main.bounds`.
    static var screenBounds: CGRect { get }

    /// A custom `UIView` set to the frame of the `screenBounds`.
    var coverView: UIView { get set }
}

public extension AppSwitchingCoverProvider {
    static var screenBounds: CGRect {
        UIScreen.main.bounds
    }
}

public struct CoverdropCoverProvider: AppSwitchingCoverProvider {
    public var coverView: UIView

    public init() {
        coverView = UIView(frame: CoverdropCoverProvider.screenBounds)
        coverView.backgroundColor = .black
    }
}
