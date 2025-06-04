import CoverDropCore
import SwiftUI

struct SelectRecipientView: View {
    @StateObject private var viewModel = SelectRecipientViewModel()
    @Binding var selectedRecipient: JournalistData?
    let recipients: MessageRecipients
    @Binding var isSelectRecipientViewOpen: Bool

    public init(isSelectRecipientViewOpen: Binding<Bool>,
                selectedRecipient: Binding<JournalistData?>,
                recipients: MessageRecipients) {
        SegmentedControlAppearance.setup()

        _isSelectRecipientViewOpen = isSelectRecipientViewOpen
        _selectedRecipient = selectedRecipient
        self.recipients = recipients
    }

    var body: some View {
        NavigationView {
            HeaderView(type: .selectRecipient) {
                VStack(alignment: .leading) {
                    Text("Select a team or journalist")
                        .textStyle(TitleStyle())
                    Text("If you know who you'd like to contact, you can add them as a recipient.")
                        .textStyle(BodyStyle())
                    VStack {
                        Picker("Recipient type", selection: $viewModel.selectedRecipientType) {
                            ForEach(SelectRecipientViewModel.RecipientType.allCases) { recipientType in
                                Text(recipientType.rawValue.capitalized)
                            }
                        }
                        .pickerStyle(.segmented)
                        journalistList(recipientType: viewModel.selectedRecipientType)
                    }
                    Spacer()
                }
                .padding(Padding.medium)
            }
        }
        .navigationViewStyle(.stack)
    }

    @ViewBuilder
    private func journalistList(recipientType: SelectRecipientViewModel.RecipientType) -> some View {
        let recipients = recipientType == .desks ? recipients.desks : recipients.journalists
        let rowHeight = recipientType == .desks ? 60 : 80
        if recipients.isEmpty {
            Text("No \(recipientType.rawValue) available").textStyle(BodyStyle()).padding(Padding.xLarge)
        } else {
            List(recipients, id: \.self) { recipient in
                switch recipientType {
                case .desks:
                    DeskRecipientItem(recipient: recipient,
                                      selectedRecipient: $selectedRecipient,
                                      isSelectRecipientViewOpen: $isSelectRecipientViewOpen)

                case .journalists:
                    JournalistRecipientItem(name: recipient.displayName,
                                            description: recipient.recipientDescription) {
                        selectedRecipient = recipient
                        $isSelectRecipientViewOpen.wrappedValue.toggle()
                    }
                }
            }
            .listStyle(.plain)
            .environment(\.defaultMinListRowHeight, CGFloat(rowHeight))
            .padding(3) // aligns items with segmented control, and adds a little spacing between the components
        }
    }
}

struct SelectRecipientView_Previews: PreviewProvider {
    static var previews: some View {
        var testRecipients = try? MessageRecipients(
            verifiedPublicKeys: PublicKeysHelper.shared.testKeys,
            excludingDefaultRecipient: false
        )
        PreviewWrapper(SelectRecipientView(isSelectRecipientViewOpen: .constant(true),
                                           selectedRecipient: .constant(testRecipients!.defaultRecipient!),
                                           recipients: testRecipients!))
        var _: ()? = testRecipients?.removeDesks()
        PreviewWrapper(SelectRecipientView(isSelectRecipientViewOpen: .constant(true),
                                           selectedRecipient: .constant(testRecipients!.defaultRecipient!),
                                           recipients: testRecipients!))
    }
}
