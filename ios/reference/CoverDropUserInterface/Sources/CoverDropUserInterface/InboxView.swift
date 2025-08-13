import CoverDropCore
import SwiftUI

struct InboxView: View {
    @ObservedObject var inboxViewModel: InboxViewModel
    @ObservedObject var conversationViewModel: ConversationViewModel
    @State private var showingDeleteAlert = false
    @State private var showingDismissalAlert = false
    @Binding var navPath: NavigationPath

    var body: some View {
        HeaderView(type: .inbox) {
            VStack {
                if let activeConversation = inboxViewModel.activeConversation {
                    activeConversationView(for: activeConversation)
                        .padding(Padding.large)
                }
                if let inactiveConversations = inboxViewModel.inactiveConversations,
                   inactiveConversations.count > 0 {
                    inactiveConversationsView(for: inactiveConversations)
                }
                Spacer()
                deleteYourMessages
                customDivider()
                footer
            }
            .navigationBarHidden(true)
            .foregroundColor(Color.StartCoverDropSessionView.foregroundColor)
        }
    }

    @ViewBuilder
    private func messagingTitle() -> Text {
        Text("Messaging with")
    }

    @ViewBuilder
    func activeConversationView(for activeConversation: ActiveConversation) -> some View {
        VStack(alignment: .leading, spacing: 0) {
            Button(action: {
                conversationViewModel.messageRecipient = activeConversation.recipient
                navPath.append(Destination.viewConversation)
            }) {
                HStack {
                    VStack(alignment: .leading, spacing: 0) {
                        messagingTitle()
                            .textStyle(GuardianHeadlineSmallTextStyle())
                            .foregroundColor(Color.InboxView.activeMessageSubHeaderColor)
                        Text(activeConversation.recipient.displayName)
                            .textStyle(TitleStyle())
                            .foregroundColor(Color.InboxView.activeMessageRecipientColor)

                            .padding([.top], Padding.medium)
                    }
                    .padding([.top, .leading, .trailing], Padding.large)
                    .padding(.bottom, Padding.small)
                    Spacer()
                    Image(systemName: "chevron.forward")
                        .resizable()
                        .fontWeight(.semibold)
                        .frame(width: 7, height: 11)
                        .foregroundColor(Color.ChevronButtonList.chevronColor)
                        .padding([.trailing], Padding.medium)
                }
            }
            customDivider()
            HStack(alignment: .top) {
                Image(systemName: "bubble.left")
                    .foregroundColor(Color.InboxView.lastMessageBubbleColor)

                Text("Last message")
                    .textStyle(MessageDetailTextStyle())
                Spacer()
                Text(activeConversation.formattedLastMessageUpdated)
                    .textStyle(MessageMetadata())
                    .padding([.top], Padding.small)
            }
            .padding([.leading, .trailing], Padding.large)
            .padding([.top], Padding.small)
            if let expiryString = inboxViewModel.activeConversation?.messages.maybeExpiryDate() {
                customDivider()
                expiredInformationText(expiryDate: expiryString)
                    .padding([.leading], Padding.medium)
            }
        }
        .background(
            RoundedRectangle(cornerRadius: CornerRadius.medium)
                .stroke(Color.InboxView.activeMessageBorderColor, lineWidth: 1)
        )
    }

    @ViewBuilder
    func inactiveConversationsView(for inactiveConversations: [InactiveConversation]) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text("Previous messages").textStyle(InlineButtonTextStyle())
                .foregroundColor(Color.InboxView.previousMessagesTitleColor)
            customDivider()
            ForEach(inactiveConversations, id: \.recipient) { conversation in
                inactiveConversationsRow(for: conversation)
            }
            customDivider()
        }
        .padding(Padding.large)
    }

    @ViewBuilder
    func inactiveConversationsRow(for inactiveConversation: InactiveConversation) -> some View {
        VStack(alignment: .leading, spacing: 0) {
            messagingTitle()
                .textStyle(GuardianHeadlineXSmallHeaderTextStyle())
                .foregroundColor(Color.InboxView.previousMessagesSubheaderColor)
            Text(inactiveConversation.recipient.displayName)
                .textStyle(GuardianHeaderTextStyle())
                .foregroundColor(Color.InboxView.previousMessageRecipientColor)
                .onTapGesture {
                    conversationViewModel.messageRecipient = inactiveConversation.recipient
                    navPath.append(Destination.viewConversation)
                }
            if let expiryDate = inactiveConversation.messages.maybeExpiryDate() {
                expiredInformationText(expiryDate: expiryDate)
            }
        }
        .padding([.top, .bottom], 10)
        .padding(.leading, Padding.large)
    }

    var deleteYourMessages: some View {
        HStack {
            Button {
                showingDeleteAlert = true
            } label: {
                HStack {
                    Image(systemName: "trash.fill")
                        .foregroundColor(Color.InboxView.deleteMessageButtonColor)
                    Text("Delete message vault")
                        .textStyle(InlineButtonTextStyle())
                        .foregroundColor(Color.InboxView.deleteMessageButtonColor)
                }
            }
            .accessibilityLabel("Delete message vault")
            .alert("Delete your message vault?",
                   isPresented: $showingDeleteAlert,
                   actions: {
                       Button("Delete everything", role: .destructive) {
                           Task {
                               try? await inboxViewModel.deleteAllMessagesAndCurrentSession(
                                   conversationViewModel: conversationViewModel
                               )
                           }
                       }
                       Button("Cancel", role: .cancel) {}
                   }, message: {
                       Text(
                           """
                           This will delete your vault and you will not be able to unlock it anymore. \
                           You won't be able to read any existing messages or responses to them. \
                           Messages you've recently sent may still be received by the journalist.
                           """
                       )
                   })
            Spacer()
        }
        .padding(.leading, Padding.large)
        .padding(.bottom, Padding.medium)
    }

    var footer: some View {
        HStack {
            Button(action: {
                navPath.append(Destination.about)
            }) {
                Text("About Secure Messaging")
                    .textStyle(InlineButtonTextStyle())
                    .foregroundColor(Color.InboxView.aboutButtonColor)
            }
            Spacer()
            Button("Leave vault") {
                showingDismissalAlert = true
            }
            .buttonStyle(XSmallFilledButtonStyle())
        }
        .padding(.leading, Padding.large)
        .padding(.trailing, Padding.small)
        .padding(.bottom, Padding.medium)
        .alert("Leaving your message vault",
               isPresented: $showingDismissalAlert,
               actions: {
                   LogoutDialogView(conversationViewModel: conversationViewModel)
               },
               message: {
                   Text(
                       """
                       This will log you out of your secure vault. Are you sure?
                       """
                   )
               })
    }

    func expiredInformationText(expiryDate: String) -> some View {
        return Text("\(Image(systemName: "info.circle.fill")) Expiring in \(expiryDate)")
            .textStyle(ExpiringMessageMetadata())
            .foregroundColor(Color.NewMessageView.messageInformationStrokeColor)
            .padding([.top], 9)
            .padding([.leading, .bottom], Padding.small)
    }
}

#Preview {
    @Previewable @State var loaded: Bool = false
    @Previewable @State var conversationViewModel: ConversationViewModel?
    @Previewable @State var inboxViewModel: InboxViewModel?
    Group {
        if loaded {
            InboxView(
                inboxViewModel: inboxViewModel!,
                conversationViewModel: conversationViewModel!,
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
            let lib = try context.getLibraryWithVerifiedKeys()
            let data = try await CoverDropServiceHelper.addTestMessagesForPreview(lib: lib)
            lib.secretDataRepository.setUnlockedDataForTesting(unlockedData: data)
            inboxViewModel = InboxViewModel(lib: lib)
            conversationViewModel = ConversationViewModel(lib: lib)
            loaded = true
        }
    }.previewFonts().environment(CoverDropUserInterfaceConfiguration(
        showAboutScreenDebugInformation: true,
        showBetaBanner: true
    ))
}
