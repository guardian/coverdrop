import SwiftUI

/// This provides a logout dialog button and action for the user when they are logging out
/// of their secure messaging session.
struct LogoutDialogView: View {
    @ObservedObject var conversationViewModel: ConversationViewModel
    var body: some View {
        Button("Log out") {
            Task {
                await conversationViewModel.clearModelDataAndLock()
            }
        }
        Button("Cancel", role: .cancel) {}
    }
}
