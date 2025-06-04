import CoverDropCore
import SwiftUI

struct DeskRecipientItem: View {
    let recipient: JournalistData
    @Binding var selectedRecipient: JournalistData?
    @Binding var isSelectRecipientViewOpen: Bool

    var body: some View {
        NavigationLink {
            let viewModel = DeskDetailViewModel(recipient: recipient)
            DeskDetailView(selectedRecipient: $selectedRecipient,
                           isSelectRecipientViewOpen: $isSelectRecipientViewOpen,
                           viewModel: viewModel)
                .navigationBarHidden(true)
        } label: {
            Text(recipient.displayName)
                .textStyle(ListItemTitleTextStyle())
        }
        .recipientItemStyle()
    }
}

struct JournalistRecipientItem: View {
    let name: String
    let description: String
    let buttonAction: () -> Void
    var body: some View {
        HStack {
            VStack(alignment: .leading, spacing: 4) {
                Text(name)
                    .textStyle(ListItemTitleTextStyle())
                Text(description)
                    .textStyle(ListItemDetailTextStyle())
            }
            Spacer()
            Button("Select") {
                buttonAction()
            }
            .buttonStyle(XSmallButtonStyle())
            .accessibilityLabel("Select \(name)")
        }
        .recipientItemStyle()
    }
}
