import CoverDropCore
import SwiftUI

struct DeskDetailView: View {
    @Environment(\.dismiss) private var dismiss
    @Binding var selectedRecipient: JournalistData?
    @Binding var isSelectRecipientViewOpen: Bool

    let viewModel: DeskDetailViewModel

    var body: some View {
        HeaderView(type: .deskDetail) {
            VStack(alignment: .leading) {
                Text(viewModel.recipient.displayName)
                    .textStyle(TitleStyle())
                Text(viewModel.recipient.recipientDescription)
                    .textStyle(BodyStyle())
                Spacer()
                VStack(alignment: .center, spacing: 0) {
                    Button("Select team") {
                        selectedRecipient = viewModel.recipient
                        isSelectRecipientViewOpen = false
                    }
                    .buttonStyle(PrimaryButtonStyle(isDisabled: false))
                    Button("Back to list") {
                        dismiss()
                    }
                    .buttonStyle(FooterButtonStyle())
                }
            }
            .padding(Padding.medium)
        }
    }
}

struct DeskDetailViewModel {
    let recipient: JournalistData
}

struct DeskDetailView_Previews: PreviewProvider {
    static var previews: some View {
        let testRecipient = try? MessageRecipients(verifiedPublicKeys: PublicKeysHelper.shared.testKeys)
            .defaultRecipient!
        let viewModel = DeskDetailViewModel(recipient: testRecipient!)
        PreviewWrapper(DeskDetailView(selectedRecipient: .constant(testRecipient),
                                      isSelectRecipientViewOpen: .constant(true),
                                      viewModel: viewModel))
    }
}
