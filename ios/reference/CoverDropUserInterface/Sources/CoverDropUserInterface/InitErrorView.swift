import CoverDropCore
import Foundation
import SwiftUI

struct InitErrorView: View {
    @Environment(CoverDropUserInterfaceConfiguration.self) var uiConfig

    let error: String
    var body: some View {
        HeaderView(type: .onboarding) {
            VStack {
                Spacer()
                Text(
                    """
                    The Secure Messaging feature is currently not available.
                    Make sure you device has a working internet connection and \
                    you are on the most recent version of the app.
                    Please restart the app and try again later.
                    """
                )
                if uiConfig.showAboutScreenDebugInformation {
                    Text(error).font(.system(size: 12, design: .monospaced)).padding(Padding.medium)
                }
                Spacer()
            }.padding(10)
                .foregroundColor(Color.StartCoverDropSessionView.foregroundColor)
        }
        .navigationBarHidden(true)
    }
}

#Preview {
    InitErrorView(error: """
    Error Domain=NSCocoaErrorDomain Code=257 "The file \
    "privateSendingQueue" couldn't be opened because you don't have permission to view it.\
    Application%20Support/ privateSendingQueue, NSUnderlyingError=0x3013f9200
    {Error Domain=NSPOSIXErrorDomai}
    """)
    .environment(CoverDropUserInterfaceConfiguration(showAboutScreenDebugInformation: true, showBetaBanner: true))
}
