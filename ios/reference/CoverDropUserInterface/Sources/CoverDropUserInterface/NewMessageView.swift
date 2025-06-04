import Combine
import CoverDropCore
import SwiftUI

struct NewMessageView: View {
    @Binding var navPath: NavigationPath
    @State var isSelectRecipientViewOpen = false
    @State private var showingDismissalAlert = false
    @State private var showingForcedSelectionAlert = false
    @FocusState private var focusedField: Field?

    // In practice, this view model's optionals should never be nil if accessed when state == .ready. Force unwrapping
    // will allow us to fail fast in the case of developer error.
    @ObservedObject var conversationViewModel: ConversationViewModel

    private enum Field: Int, CaseIterable {
        case message
    }

    init(conversationViewModel: ConversationViewModel, navPath: Binding<NavigationPath>) {
        _navPath = navPath
        self.conversationViewModel = conversationViewModel
    }

    var body: some View {
        switch conversationViewModel.state {
        case .initial, .loading, .sending:
            ProgressView()
        case .ready:
            newMessage()
        case let .error(error):
            Text("Error: \(error)")
        }
    }

    private func newMessage() -> some View {
        HeaderView(
            type: .newConversation,
            dismissAction: { showingDismissalAlert = true }
        ) {
            CraftMessageBannerView(action: {
                navPath.append(Destination.help(contentVariant: .craftMessage))
            })

            VStack(alignment: .leading) {
                let titleText = "What do you want to share with us?"
                Text(titleText)
                    .textStyle(TitleStyle())

                chooseRecipient()
                messageCompose()
                Spacer()
                messageSend()
            }
            .padding(Padding.large)
            .foregroundColor(Color.NewMessageView.foregroundColor)
            .onTapGesture {
                // This closes the keyboard when tapping outside of the message text field
                focusedField = nil
            }.navigationBarHidden(true)
            .navigationBarTitle(Text(""))
            .alert("Leaving your message vault",
                   isPresented: $showingDismissalAlert,
                   actions: {
                       LogoutDialogView(conversationViewModel: conversationViewModel)
                   },
                   message: {
                       Text(
                           """
                           This will log you out of your secure vault. Your unfinished message will not be sent. \
                           Are you sure?
                           """
                       )
                   })
        }
    }

    @ViewBuilder
    func chooseRecipient() -> some View {
        VStack(alignment: .leading, spacing: 0) {
            Text("Select a journalist or team")
                .textStyle(FormLabelTextStyle())

            Text("Your message is reviewed by journalists")
                .textStyle(BodyStyle())
        }

        ZStack(alignment: .trailing) {
            let forcedSingleRecipient = conversationViewModel.recipients?.forcedPreselectedRecipient()

            if let messageRecipient = conversationViewModel.messageRecipient {
                Text("\(messageRecipient.displayName)")
                    .textStyle(SelectRecipientTextStyle())
                    .disableAutocorrection(true)
                    .autocapitalization(.none)
                    .accessibilityIdentifier("Selected Recipient is \(messageRecipient.displayName)")
                    .onTapGesture {
                        if forcedSingleRecipient != nil {
                            DispatchQueue.main.async {
                                showingForcedSelectionAlert = true
                            }
                        }
                    }
                    .alert(
                        "At the current time you can only contact a single Guardian recipient.",
                        isPresented: $showingForcedSelectionAlert
                    ) {
                        Button("Dismiss", role: .cancel) {}
                    }
            } else {
                Text("No recipient selected")
                    .textStyle(SelectRecipientTextStyle())
                    .disableAutocorrection(true)
                    .autocapitalization(.none)
                    .accessibilityIdentifier("Selected Recipient")
            }
            if let recipients = conversationViewModel.recipients, forcedSingleRecipient == nil {
                Button(action: {
                    isSelectRecipientViewOpen = true
                }, label: {
                    HStack(alignment: .center) {
                        Image(systemName: "pencil").resizable().frame(width: 12, height: 12)
                        Text("Change recipient").textStyle(InlineButtonTextStyle())
                    }
                }).sheet(isPresented: $isSelectRecipientViewOpen) {
                    SelectRecipientView(isSelectRecipientViewOpen: $isSelectRecipientViewOpen,
                                        selectedRecipient: $conversationViewModel.messageRecipient,
                                        recipients: recipients)
                }
                .accessibilityIdentifier("Select a Recipient")
                .padding([.trailing], Padding.medium)
            }
        }
        .padding([.bottom], Padding.medium)
    }

    @ViewBuilder
    func messageSend() -> some View {
        Spacer()
        Rectangle().fill(Color.NewMessageView.messageToLongErrorColor)
            .frame(height: 2)
            .opacity(conversationViewModel.messageIsTooLong ? 1 : 0)
        ZStack {
            // Send button
            Button("Send message") {
                Task {
                    do {
                        try await conversationViewModel.sendMessage()
                        conversationViewModel.clearMessage()
                    } catch {
                        // We drop errors sliently to avoid giving away any user interaction in logs
                    }
                }
            }
            .buttonStyle(PrimaryButtonStyle(isDisabled: conversationViewModel.sendButtonDisabled))
            .disabled(conversationViewModel.sendButtonDisabled)
        }
    }

    @ViewBuilder
    func messageCompose() -> some View {
        VStack(alignment: .leading, spacing: 0) {
            Text("What's your message?")
                .textStyle(FormLabelTextStyle())
            Text("See ‘Compose your first message’ for advice")
                .textStyle(FormLabelSubtitleTextStyle())
        }
        VStack(alignment: .leading, spacing: 0) {
            MessageLengthProgressView(
                messageLengthProgressPercentage: conversationViewModel.messageLengthProgressPercentage
            )
            TextEditor(text: $conversationViewModel.message)
                .style(ComposeMessageTextStyle())
                .accessibilityIdentifier("Compose your message")
                .frame(minHeight: 80, maxHeight: .greatestFiniteMagnitude)
                .focused($focusedField, equals: .message)
        }
    }
}

#Preview {
    @Previewable @State var loaded: Bool = false
    @Previewable @State var conversationViewModel: ConversationViewModel?

    Group {
        if loaded {
            NewMessageView(conversationViewModel: conversationViewModel!, navPath: .constant(NavigationPath()))
        } else {
            Group {
                LoadingView()
            }
        }
    }.onAppear {
        Task {
            let context = IntegrationTestScenarioContext(scenario: .minimal, config: StaticConfig.devConfig)
            let lib = try context.getLibraryWithVerifiedKeys()
            conversationViewModel = ConversationViewModel(lib: lib)
            loaded = true
        }
    }
    .previewFonts()
    .environment(CoverDropUserInterfaceConfiguration(showAboutScreenDebugInformation: true, showBetaBanner: true))
}
