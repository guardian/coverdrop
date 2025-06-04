import CoverDropCore
import Foundation
import SwiftUI

struct AsyncActionButton: View {
    var buttonText: String
    var isInProgress: Bool

    var action: () async throws -> Void

    var body: some View {
        if isInProgress {
            Button(action: {}, label: {
                ProgressView().progressViewStyle(.circular).tint(.black)
            })
            .buttonStyle(PrimaryButtonStyle(isDisabled: true))

        } else {
            Button(buttonText) {
                Task {
                    try await action()
                }
            }
            .keyboardShortcut(.defaultAction)
            .buttonStyle(PrimaryButtonStyle(isDisabled: false))
        }
    }
}
