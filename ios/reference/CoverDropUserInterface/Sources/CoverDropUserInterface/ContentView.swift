import CoverDropCore
import Foundation
import SwiftUI

enum Destination: Equatable, Hashable {
    case inbox
    case newConversation
    case viewConversation
    case about
    case home
    case onboarding
    case newPassphrase
    case login
    case messageSent
    case deskDetail
    case selectRecipient
    case help(contentVariant: HelpScreenContent)
}

struct ContentView: View {
    var config: CoverDropConfig
    var uiConfig: CoverDropUserInterfaceConfiguration
    @ObservedObject var coverDropService: CoverDropService = .shared
    @State private var showingScreenshotDetectedAlert = false

    var body: some View {
        Group {
            switch coverDropService.state {
            case .notInitialized:
                InitErrorView(error: "Secure messaging unavailable")
            case .initializing:
                LoadingView()
            case let .initialized(lib: lib):
                ReadyView(
                    lib: lib,
                    secretDataRepository: lib.publishedSecretDataRepository,
                    conversationViewModel: ConversationViewModel(lib: lib)
                )
            case let .failedToInitialize(reason: reason):
                InitErrorView(error: "error: \(reason)")
            }
        }.environment(uiConfig)
            .alert(
                """
                You took a screenshot.
                These screenshots can appear in your photo library and can be uploaded to the cloud, \
                so this may be a security risk.
                """,
                isPresented: $showingScreenshotDetectedAlert
            ) {
                Button("OK", role: .cancel) {
                    showingScreenshotDetectedAlert = false
                }
            }

            .onReceive(NotificationCenter.default
                .publisher(for: UIApplication.userDidTakeScreenshotNotification)) { _ in
                    DispatchQueue.main.async {
                        showingScreenshotDetectedAlert = true
                    }
            }
    }
}

struct ReadyView: View {
    @ObservedObject var securitySuite = SecuritySuite.shared
    @ObservedObject var lib: CoverDropLibrary
    // Note that if you need to check the state of an enum, it needs it to be
    // a top level object, so we cannot observe lib.secretDataRepository, but can
    // observe secretDataRepository itself.
    @ObservedObject var secretDataRepository: SecretDataRepository
    @ObservedObject var conversationViewModel: ConversationViewModel
    @State var postLoginDestination: UnlockedSecretDataDestination = .newConversation

    var body: some View {
        if !securitySuite.getEffectiveViolationsSet().isEmpty {
            SecurityAlert()
        } else if case .unlockedSecretData = secretDataRepository.secretData {
            InboxStateView(
                secretDataRepository: lib.publishedSecretDataRepository,
                conversationViewModel: conversationViewModel,
                lib: lib,
                postLoginDestination: postLoginDestination
            )
        } else {
            NonLoggedInNavigationView(
                lib: lib,
                postLoginDestination: $postLoginDestination
            )
        }
    }
}

struct NonLoggedInNavigationView: View {
    @State private var navPath = NavigationPath()
    @ObservedObject var lib: CoverDropLibrary
    @Binding var postLoginDestination: UnlockedSecretDataDestination
    @StateObject var userNewSessionViewModel: UserNewSessionViewModel

    init(
        navPath: NavigationPath = NavigationPath(),
        lib: CoverDropLibrary,
        postLoginDestination: Binding<UnlockedSecretDataDestination>
    ) {
        self.navPath = navPath
        self.lib = lib
        _postLoginDestination = postLoginDestination
        _userNewSessionViewModel = StateObject(wrappedValue: UserNewSessionViewModel(lib: lib))
    }

    var body: some View {
        NavigationStack(path: $navPath) {
            Group {
                StartCoverDropSessionView(
                    publicDataRepository: lib.publishedPublicDataRepository, navPath: $navPath
                )
            }.navigationDestination(for: Destination.self) { destination in
                switch destination {
                case .about:
                    AboutCoverDropView(
                        navPath: $navPath,
                        viewModel: AboutCoverDropViewModel(lib: lib)
                    )
                case .onboarding:
                    OnboardingView(navPath: $navPath)
                case .newPassphrase:
                    UserNewSessionView(
                        navPath: $navPath,
                        passphraseWordCount: lib.config.passphraseWordCount,
                        viewModel: userNewSessionViewModel
                    )
                case .login:
                    UserContinueSessionView(
                        navPath: $navPath,
                        viewModel: UserContinueSessionViewModel(lib: lib)
                    ).onAppear {
                        $postLoginDestination.wrappedValue = .login
                    }
                case .newConversation:
                    UserContinueSessionView(
                        navPath: $navPath,
                        viewModel: UserContinueSessionViewModel(lib: lib)
                    ).onAppear {
                        $postLoginDestination.wrappedValue = .newConversation
                    }
                case .home:
                    StartCoverDropSessionView(
                        publicDataRepository: lib.publishedPublicDataRepository, navPath: $navPath
                    )
                case let .help(contentVariant):
                    // This switch might appear unneccessary. However, the Navigation object will not correctly
                    // invalid the HelpView if we do not split into separate code paths...
                    switch contentVariant {
                    case .craftMessage:
                        HelpView(contentVariant: .craftMessage, navPath: $navPath)
                    case .faq:
                        HelpView(contentVariant: .faq, navPath: $navPath)
                    case .howSecureMessagingWorks:
                        HelpView(contentVariant: .howSecureMessagingWorks, navPath: $navPath)
                    case .keepingPassphraseSafe:
                        HelpView(contentVariant: .keepingPassphraseSafe, navPath: $navPath)
                    case .privacyPolicy:
                        HelpView(contentVariant: .privacyPolicy, navPath: $navPath)
                    case .replyExpectations:
                        HelpView(contentVariant: .replyExpectations, navPath: $navPath)
                    case .sourceProtection:
                        HelpView(contentVariant: .sourceProtection, navPath: $navPath)
                    case .whyWeMadeSecureMessaging:
                        HelpView(contentVariant: .whyWeMadeSecureMessaging, navPath: $navPath)
                    }
                case _:
                    StartCoverDropSessionView(
                        publicDataRepository: lib.publishedPublicDataRepository, navPath: $navPath
                    )
                }
            }
        }
    }
}
