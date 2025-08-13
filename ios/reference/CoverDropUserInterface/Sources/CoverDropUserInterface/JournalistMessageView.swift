import CoverDropCore
import SwiftUI

enum MessageError: Error {
    case failedToLoad
}

struct JournalistMessageView: View {
    @ObservedObject var lib: CoverDropLibrary
    @ObservedObject var inboxViewModel: InboxViewModel
    @ObservedObject var conversationViewModel: ConversationViewModel

    /// This is required for the view to re-render when items are popped from the PSQ, which can
    /// cause messages go from `pending` to `sent`.
    @ObservedObject var privateSendingQueue = PrivateSendingQueueRepository.shared

    @Binding var navPath: NavigationPath

    // by default we want to make the user have to choose to send another message
    @State var alreadySentMessage: Bool = false

    var journalist: JournalistData

    init(
        journalist: JournalistData,
        conversationViewModel: ConversationViewModel,
        lib: CoverDropLibrary,
        navPath: Binding<NavigationPath>
    ) {
        self.journalist = journalist
        self.conversationViewModel = conversationViewModel
        self.lib = lib
        inboxViewModel = InboxViewModel(lib: lib)
        _navPath = navPath
    }

    var body: some View {
        HeaderView(type: .viewConversation, dismissAction: {
            if !navPath.isEmpty {
                navPath.removeLast()
            }
        }) {
            VStack(alignment: .leading, spacing: 0) {
                let recipientOpt = conversationViewModel.messageRecipient
                if let recipient = recipientOpt {
                    messageListView()
                    messageComposeView(recipient: recipient)
                }
            }
            .navigationBarTitle("Secure chat with \(journalist.displayName)", displayMode: .inline)
            .navigationBarHidden(true)
        }
    }

    private func messageComposeView(recipient: JournalistData) -> some View {
        return VStack {
            let expired = isCurrentKeyExpired(recipient: recipient)
            let inactive = !self.conversationViewModel
                .isCurrentConversationActive(maybeActiveConversation: inboxViewModel.activeConversation)
            let isMostRecentMessageFromUser = self.conversationViewModel.isMostRecentMessageFromUser()

            if expired {
                expiredKeysMessage(recipient: recipient)
            } else if inactive {
                viewingInactiveConversation()
            } else if isMostRecentMessageFromUser && !alreadySentMessage {
                chooseToSentAnotherMessage()
            } else {
                messageSendView()
            }
        }
    }

    func isCurrentKeyExpired(recipient: JournalistData) -> Bool {
        if let verifiedKeys = try? lib.publicDataRepository.getVerifiedKeys(),
           let currentKey = verifiedKeys.getLatestMessagingKey(journalistId: recipient.recipientId) {
            return currentKey.isExpired(now: DateFunction.currentTime())
        }
        return false
    }

    private func messageListView() -> some View {
        ScrollViewReader { scrollViewProxy in
            GeometryReader { geometry in
                ScrollView(.vertical, showsIndicators: false) {
                    VStack(alignment: .leading) {
                        VStack(alignment: .center) {
                            if let recipient = self.conversationViewModel.messageRecipient {
                                let headerMessage = "This is a secure conversation with \(recipient.displayName)"
                                Text(
                                    "\(Image(systemName: "lock.fill")) \(headerMessage)"
                                ).textStyle(BodyStyle())
                                    .bold()
                            }
                        }
                        switch self.conversationViewModel.state {
                        case .initial, .loading, .ready, .sending:
                            switch lib.secretDataRepository.getSecretData() {
                            case let .unlockedSecretData(unlockedData: unlockedData):
                                messageListContents(
                                    unlockedData: unlockedData,
                                    scrollViewProxy: scrollViewProxy
                                )
                            case _:
                                EmptyView()
                            }
                        case let .error(error):
                            Text("Error: \(error)")
                        }
                    }
                }.padding(Padding.medium)
                    .background(Color.JournalistNewMessageView.messageListBackgroundColor)
                    .foregroundColor(Color.JournalistNewMessageView.messageListForegroundColor)
                    .frame(maxHeight: geometry.size.height)
                // This spacer keeps the middle pane in full height
                Spacer()
            }
        }
    }

    func messageListContents(unlockedData: UnlockedSecretData, scrollViewProxy: ScrollViewProxy) -> some View {
        Group {
            let count = conversationViewModel.currentConversationForUi.count
            ForEach(conversationViewModel.currentConversationForUi.indices, id: \.self) { index in
                if let message = conversationViewModel.currentConversationForUi[index] {
                    MessageView(message: message, id: index)
                }
            }.onChange(of: unlockedData.messageMailbox.count) {
                scrollToLastMessage(scrollViewProxy: scrollViewProxy, count: count)
            }.onAppear {
                scrollToLastMessage(scrollViewProxy: scrollViewProxy, count: count)
            }
            if conversationViewModel.isMostRecentMessageFromUser() {
                if conversationViewModel
                    .isMostRecentMessagePending() {
                    Text(
                        """
                        Your message is now queued up for secure transmission \
                        within routine Guardian app activity. \
                        We recommend that you exit Secure Messaging and wait \
                        for a response before you send another message.
                        """
                    ).textStyle(UserNotificationTextStyle())
                } else {
                    Text(
                        """
                        Your message has been sent. \
                        We recommend that you exit Secure Messaging and \
                        wait for a response before you send another message.
                        """
                    )
                    .textStyle(UserNotificationTextStyle())
                }
            }
        }
    }

    func chooseToSentAnotherMessage() -> some View {
        return VStack {
            Button("Send a new message") {
                self.alreadySentMessage = true
            }.buttonStyle(SecondaryButtonStyle(isDisabled: false))
                .accessibilityLabel("Send a new message")
        }.foregroundColor(Color.ComposeMessageTextStyle.foregroundColor)
            .padding(Padding.medium)
    }

    func viewingInactiveConversation() -> some View {
        return InformationView(
            viewType: .info,
            title: "This conversation has been closed",
            message: "Go to your active conversation to send a message."
        ).padding(Padding.medium)
    }

    func expiredKeysMessage(recipient: JournalistData) -> some View {
        return InformationView(
            viewType: .info,
            title: "\(recipient.displayName) is currently unavailable.",
            message: "Check your internet connection or try again later."
        )
        .padding(Padding.medium)
    }

    func messageSendView() -> some View {
        return VStack(alignment: .leading, spacing: 0) {
            MessageLengthProgressView(
                messageLengthProgressPercentage: conversationViewModel.messageLengthProgressPercentage
            )
            .padding([.horizontal], Padding.medium)
            TextEditor(text: $conversationViewModel.message)
                .style(ComposeMessageTextStyle())
                .frame(minHeight: 60, maxHeight: 80)
                .padding(Padding.medium)
                .accessibilityLabel("Compose your message")

            Button("Send") {
                Task {
                    try? await conversationViewModel.sendMessage()
                    conversationViewModel.clearMessage()
                    self.alreadySentMessage = false
                }
            }.disabled(conversationViewModel.sendButtonDisabled)
                .buttonStyle(PrimaryButtonStyle(isDisabled: conversationViewModel.sendButtonDisabled))
                .padding([.horizontal], Padding.medium)
                .accessibilityLabel("Send")
        }
    }

    private func scrollToLastMessage(scrollViewProxy: ScrollViewProxy, count: Int) {
        switch lib.secretDataRepository.getSecretData() {
        case let .unlockedSecretData(unlockedData: unlockedData):
            let unwrappedId = count - 1
            withAnimation {
                scrollViewProxy.scrollTo(unwrappedId, anchor: .bottom)
            }
        case _:
            return
        }
    }
}

#Preview {
    @Previewable @State var loaded: Bool = false
    @Previewable @State var lib: CoverDropLibrary?
    @Previewable @State var conversationViewModel: ConversationViewModel?
    @Previewable @State var inboxViewModel: InboxViewModel?
    @Previewable @State var journalist: JournalistData?

    Group {
        if loaded {
            JournalistMessageView(
                journalist: journalist!,
                conversationViewModel: conversationViewModel!,
                lib: lib!,
                navPath: .constant(NavigationPath())
            )

        } else {
            Group {
                LoadingView()
            }
        }
    }.onAppear {
        Task {
            let context = IntegrationTestScenarioContext(scenario: .minimal, config: StaticConfig.devConfig)
            let library = try context.getLibraryWithVerifiedKeys()
            let data = try await CoverDropServiceHelper.addTestMessagesForPreview(lib: library)
            library.secretDataRepository.setUnlockedDataForTesting(unlockedData: data)
            inboxViewModel = InboxViewModel(lib: library)
            conversationViewModel = ConversationViewModel(lib: library)
            lib = library
            if let testDefaultJournalist = PublicKeysHelper.shared.testDefaultJournalist {
                journalist = testDefaultJournalist
                conversationViewModel?.messageRecipient = journalist
                loaded = true
            }
        }
    }.previewFonts()
        .environment(CoverDropUserInterfaceConfiguration(showAboutScreenDebugInformation: true, showBetaBanner: true))
}
