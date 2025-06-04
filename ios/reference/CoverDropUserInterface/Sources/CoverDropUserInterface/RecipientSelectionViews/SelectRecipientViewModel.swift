import CoverDropCore
import SwiftUI

@MainActor
class SelectRecipientViewModel: ObservableObject {
    enum RecipientType: String, CaseIterable, Identifiable {
        case desks
        case journalists
        var id: Self { self }
    }

    @Published var selectedRecipientType: RecipientType = .desks
}
