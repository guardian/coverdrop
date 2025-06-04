import Combine
import CoverDropCore
import Foundation
import SwiftUI

struct InboxStateView: View {
    @State private var loggedInNavPath = NavigationPath()
    @ObservedObject var secretDataRepository: SecretDataRepository
    @StateObject var conversationViewModel: ConversationViewModel
    @ObservedObject var lib: CoverDropLibrary
    var postLoginDestination: UnlockedSecretDataDestination

    var body: some View {
        if case let .unlockedSecretData(unlockedData: secretData) = secretDataRepository.secretData {
            UnlockedSecretDataStateSwitcher(
                loggedInNavPath: $loggedInNavPath, unlockedSecretData: secretData,
                conversationViewModel: conversationViewModel,
                lib: lib,
                inboxViewModel: InboxViewModel(lib: lib),
                postLoginDestination: postLoginDestination
            )
        }
    }
}

enum UnlockedSecretDataDestination {
    case inbox, newConversation, login
}

// This view chooses the view to display when the user is logged in.
// This exta level of indirection is needed to allow us to observe changes in a value that is wrapped in
// an enum state, so we first need to establish we are in the `.unlockedSecretData` enum variant, and then pass
// this `UnlockedSecretData` into a new view so that we can observe changes to the mailbox
struct UnlockedSecretDataStateSwitcher: View {
    @Binding var loggedInNavPath: NavigationPath
    @ObservedObject var unlockedSecretData: UnlockedSecretData
    @ObservedObject var conversationViewModel: ConversationViewModel
    @ObservedObject var lib: CoverDropLibrary
    @ObservedObject var inboxViewModel: InboxViewModel
    var postLoginDestination: UnlockedSecretDataDestination

    var body: some View {
        NavigationStack(path: $loggedInNavPath) {
            Group {
                if unlockedSecretData.messageMailbox.isEmpty {
                    NewMessageView(
                        conversationViewModel: conversationViewModel,
                        navPath: $loggedInNavPath
                    )
                } else {
                    if case .newConversation = postLoginDestination {
                        // This scenario is when the user has just sent their first message for this session (after
                        // the new message flow)
                        MessageSentView(
                            lib: lib,
                            conversationViewModel: conversationViewModel,
                            navPath: $loggedInNavPath
                        )
                    } else if case .inbox = postLoginDestination {
                        // This scenario is when the user has logged in
                        InboxView(
                            inboxViewModel: inboxViewModel,
                            conversationViewModel: conversationViewModel,
                            navPath: $loggedInNavPath
                        )
                    } else {
                        InboxView(
                            inboxViewModel: inboxViewModel,
                            conversationViewModel: conversationViewModel,
                            navPath: $loggedInNavPath
                        )
                    }
                }
            }
            .navigationDestination(for: Destination.self) { destination in
                switch destination {
                case .viewConversation:
                    if conversationViewModel.messageRecipient != nil {
                        // This scenario is viewing an existing conversation
                        JournalistMessageView(
                            journalist: conversationViewModel.messageRecipient!,
                            conversationViewModel: conversationViewModel,
                            lib: lib,
                            navPath: $loggedInNavPath
                        )
                    } else {
                        InboxView(
                            inboxViewModel: inboxViewModel,
                            conversationViewModel: conversationViewModel,
                            navPath: $loggedInNavPath
                        )
                    }
                case .inbox:
                    InboxView(
                        inboxViewModel: inboxViewModel,
                        conversationViewModel: conversationViewModel,
                        navPath: $loggedInNavPath
                    )
                case .about:
                    AboutCoverDropView(
                        navPath: $loggedInNavPath,
                        viewModel: AboutCoverDropViewModel(lib: lib)
                    )
                case let .help(contentVariant):
                    // This switch might appear unneccessary. However, the Navigation object will not correctly
                    // invalid the HelpView if we do not split into separate code paths...
                    switch contentVariant {
                    case .craftMessage:
                        HelpView(contentVariant: .craftMessage, navPath: $loggedInNavPath)
                    case .faq:
                        HelpView(contentVariant: .faq, navPath: $loggedInNavPath)
                    case .howSecureMessagingWorks:
                        HelpView(contentVariant: .howSecureMessagingWorks, navPath: $loggedInNavPath)
                    case .keepingPassphraseSafe:
                        HelpView(contentVariant: .keepingPassphraseSafe, navPath: $loggedInNavPath)
                    case .privacyPolicy:
                        HelpView(contentVariant: .privacyPolicy, navPath: $loggedInNavPath)
                    case .replyExpectations:
                        HelpView(contentVariant: .replyExpectations, navPath: $loggedInNavPath)
                    case .sourceProtection:
                        HelpView(contentVariant: .sourceProtection, navPath: $loggedInNavPath)
                    case .whyWeMadeSecureMessaging:
                        HelpView(contentVariant: .whyWeMadeSecureMessaging, navPath: $loggedInNavPath)
                    }
                case _:
                    Text("Error: Unsupported destination")
                }
            }
        }
    }
}
