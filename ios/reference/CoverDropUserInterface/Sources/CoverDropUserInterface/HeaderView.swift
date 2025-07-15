import CoverDropCore
import Foundation
import SwiftUI

struct HeaderView<Content: View>: View {
    let content: Content
    let type: Destination
    @State private var showingScreenshotDetectedAlert = false
    @State private var showingBetaBannerAlert = false

    @AppStorage("coverDropEnabledRemotely", store: UserDefaults(suiteName: "coverdrop.theguardian.com")!)
    private var isCoverDropEnabled: Bool = false

    @State private var showPopover: Bool = false

    /// An optional closure to allow a view to implement its own dismissal logic. If `nil`, the parent view will be
    /// dismissed when the back button is pressed.
    let dismissAction: (() -> Void)?

    @Environment(\.presentationMode) var presentation: Binding<PresentationMode>
    @Environment(CoverDropUserInterfaceConfiguration.self) var uiConfig

    init(
        type: Destination,
        dismissAction: (() -> Void)? = nil,
        @ViewBuilder _ content: () -> Content
    ) {
        self.content = content()
        self.type = type
        self.dismissAction = dismissAction
    }

    var body: some View {
        ZStack {
            Color.HeaderView.fillColor
                .ignoresSafeArea(.all)
            VStack(spacing: 0) {
                HStack(spacing: 0) {
                    if type != .inbox {
                        backButtonView()
                    }
                    #if DEBUG
                        Spacer()
                        toggleDevMenu()
                    #endif
                    Spacer() // This pushed the button to the left corner
                    logoView()
                }
                .background(Color.HeaderView.backgroundColor)
                .padding(0)
                .transition(.move(edge: .top))
                GeometryReader { geometry in
                    ScrollView(.vertical, showsIndicators: true) {
                        VStack(spacing: 0) {
                            customDivider()
                            if uiConfig.showBetaBanner && type != .newPassphrase && type != .messageSent {
                                BetaBannerView(
                                    showBetaBannerAlert: $showingBetaBannerAlert
                                )
                                .transition(.move(edge: .top))
                                .alert(
                                    "Secure Messaging: public test",
                                    isPresented: $showingBetaBannerAlert
                                ) {
                                    Button("OK", role: .cancel) {}
                                    Button("Hide warning", role: .destructive) {
                                        uiConfig.showBetaBanner = false
                                    }
                                } message: {
                                    Text("""
                                    Feel free to try out this new way to contact Guardian reporters. \
                                    However, during this test period we can't guarantee that all messages \
                                    will be read. You may want to consider the alternatives described in \
                                    theguardian.com/tips.

                                    If you don't need to stay anonymous we'd welcome feedback at userhelp@theguardian.com.
                                    """)
                                }
                            }

                            content
                        }.frame(minHeight: geometry.size.height)
                    }
                }
            }
        }
    }

    func logoView() -> some View {
        Image(.logo)
            .resizable()
            .aspectRatio(contentMode: .fit)
            .frame(height: 40, alignment: .topLeading)
            .padding([.trailing], Padding.large)
            .padding([.vertical], Padding.medium)
    }

    func toggleDevMenu() -> some View {
        VStack {
            Button("Dev Settings") {
                showPopover = true
            }.buttonStyle(.bordered)
                .popover(isPresented: $showPopover) {
                    VStack(alignment: .leading, spacing: 20) {
                        Text("Preferences")
                            .font(.headline)

                        Toggle(
                            "CoverDrop enabled",
                            isOn: $isCoverDropEnabled
                        ).accessibilityIdentifier("toggleCoverDropEnabledButton")

                        Button("Close") {
                            showPopover = false
                        }.buttonStyle(.bordered).accessibilityIdentifier("closeDevMenuButton")
                    }
                    .frame(width: 200)
                }
        }
        .accessibilityIdentifier("toggleDevMenuButton")
        .accessibilityElement(children: .contain)
    }

    func backButtonView() -> some View {
        let (backButtonImageName, backButtonText) = backButtonInfo()
        return Button(action: {
            // This will navigate a view hierarchy back to the previous screen,
            // or close a modal window if its open.
            guard let dismissAction else { presentation.wrappedValue.dismiss(); return }
            dismissAction()
        }) {
            Image(systemName: backButtonImageName)
                .resizable()
                .frame(width: 25, height: 25)
                .padding(Padding.large)
                .foregroundColor(Color.HeaderView.arrowColor)
        }
        .buttonStyle(.plain)
        .accessibilityIdentifier(backButtonText)
    }

    /// This maps the page type to the correct descriptive text and back button icon
    func backButtonInfo() -> (text: String, imageName: String) {
        switch type {
        case .login, .viewConversation, .onboarding, .about,
             .deskDetail, .selectRecipient, .newPassphrase, .help:
            return ("arrow.backward", "Go Back")
        case .home:
            return ("xmark", "Close Secure Messaging")
        case .inbox, .messageSent, .newConversation:
            return ("xmark", "Log out of vault")
        }
    }
}

struct HeaderView_Previews: PreviewProvider {
    static var previews: some View {
        PreviewWrapper(HeaderView(type: .login) {
            EmptyView()
        }).environment(CoverDropUserInterfaceConfiguration(showAboutScreenDebugInformation: true, showBetaBanner: true))
    }
}
