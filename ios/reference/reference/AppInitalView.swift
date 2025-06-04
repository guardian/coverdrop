import CoverDropCore
import CoverDropUserInterface
import Foundation
import SwiftUI

public struct AppInitalView: View {
    @State private var showCoverDropView = false
    var appDelegate: AppDelegate {
        UIApplication.shared.delegate as! AppDelegate
    }

    public let uiConfig = CoverDropUserInterfaceConfiguration(
        showAboutScreenDebugInformation: true,
        showBetaBanner: true
    )

    public var body: some View {
        Button("Open CoverDrop") {
            showCoverDropView.toggle()
        }.fullScreenCover(isPresented: $showCoverDropView) {
            CoverDropUserInterface.initialView(
                config: appDelegate.config,
                uiConfig: uiConfig
            )
        }
    }
}
