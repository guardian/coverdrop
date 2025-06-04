import Combine
import CoverDropCore
import CryptoKit
import Foundation
import SwiftUI

enum ContinueSessionError: Error {
    case missingWords
    case misspeltWords
    case failedToUnlock
}

extension ContinueSessionError: LocalizedError {
    public var errorDescription: String? {
        switch self {
        case .missingWords:
            return "Please fill in all passphrase words."
        case .misspeltWords:
            return "The passphrase cannot be right because it contains words that are not in the word list."
        case .failedToUnlock:
            return """
            Failed to open message vault. Either you haven\'t set up a vault, or the passphrase was wrong. \
            Please re-enter the passphrase or go back to set up a vault.
            """
        }
    }
}

struct UserContinueSessionView: View {
    @Binding var navPath: NavigationPath
    @ObservedObject var viewModel: UserContinueSessionViewModel
    @FocusState private var focusedField: PasswordField?

    var body: some View {
        return HeaderView(type: .login, dismissAction: {
            navPath.isEmpty ? () : navPath.removeLast()
        }) {
            VStack(alignment: .leading) {
                switch viewModel.state {
                case .enter:
                    enteringPassphraseView()
                case .unlocking:
                    unlockingStorageView()
                }
            }
            .padding([.horizontal, .top], Padding.large)
            .padding([.bottom], Padding.medium)
            .foregroundColor(Color.StartCoverDropSessionView.foregroundColor)

            tertiaryButton(
                action: {
                    navPath.isEmpty ? () : navPath.removeLast()
                    navPath.append(Destination.onboarding)
                },
                text: "I do not have a passphrase yet"
            )

        }.navigationBarHidden(true)
            .onTapGesture {
                focusedField = nil
            }
    }

    func unlockingStorageView() -> some View {
        VStack {
            Spacer()
            ProgressView()
                .progressViewStyle(CircularProgressViewStyle())
                .tint(.white)
                .padding(Padding.large)
            Text("Unlocking secure vault… please wait. On older phones this can take a while…")
                .textStyle(BodyStyle())
            Spacer()
        }
    }

    func enteringPassphraseView() -> some View {
        VStack(alignment: .leading) {
            Text("Enter passphrase")
                .textStyle(TitleStyle(bottomPadding: Padding.medium))
            Text("If you previously set up a secure message vault, please enter your passphrase.")
                .textStyle(BodyStyle())
                .padding(.bottom, Padding.medium)
                .fixedSize(horizontal: false, vertical: true)

            if viewModel.error != nil {
                Text(viewModel.error?.localizedDescription ?? "")
                    .textStyle(FormErrorTextStyle())
                    .padding(.bottom, Padding.medium)
                    .fixedSize(horizontal: false, vertical: true)
            }

            PassphraseFormView(
                wordCount: viewModel.wordCount,
                words: $viewModel.enteredWords,
                wordVisible: $viewModel.wordVisible,
                wordInvalid: viewModel.invalidWords,
                passwordFieldFocus: $focusedField
            )

            HStack {
                Spacer()
                if viewModel.wordVisible.allSatisfy({ $0 }) {
                    Button("Hide all") {
                        viewModel.hideAll()
                    }.buttonStyle(InlineButtonStyle())
                } else {
                    Button("Show all") {
                        viewModel.showAll()
                    }.buttonStyle(InlineButtonStyle())
                }
            }

            Spacer()

            Button("Confirm passphrase") {
                Task {
                    await viewModel.unlockStorage()
                }
            }.buttonStyle(PrimaryButtonStyle(isDisabled: false))
        }
    }
}

@MainActor class UserContinueSessionViewModel: ObservableObject {
    enum State {
        case enter
        case unlocking
    }

    var lib: CoverDropLibrary
    var wordCount: Int { lib.config.passphraseWordCount }
    var validPrefixes = Set<String>()

    @Published var state: State = .enter
    @Published var error: Error?

    @Published var enteredWords: [String] = []
    @Published var wordVisible: [Bool] = []

    var invalidWords: [Bool] {
        // return `true` for each non-empty word that is not a valid prefix
        enteredWords.map { word in !word.isEmpty && !validPrefixes.contains(word) }
    }

    init(lib: CoverDropLibrary) {
        self.lib = lib
        enteredWords = Array(repeating: "", count: wordCount)
        wordVisible = Array(repeating: true, count: wordCount)
        validPrefixes = PasswordGenerator.shared.generatePrefixes()
    }

    func showAll() {
        wordVisible = Array(repeating: true, count: wordCount)
    }

    func hideAll() {
        wordVisible = Array(repeating: false, count: wordCount)
    }

    func unlockStorage() async {
        guard case .enter = state else {
            return
        }

        if enteredWords.contains(where: { $0.isEmpty }) {
            error = ContinueSessionError.missingWords
            return
        }

        let enteredPassphrase = enteredWords.joined(separator: " ")

        guard let validatedPassphrase = try? PasswordGenerator.checkValid(passwordInput: enteredPassphrase) else {
            error = ContinueSessionError.misspeltWords
            return
        }

        state = .unlocking
        do {
            try await lib.secretDataRepository.unlock(passphrase: validatedPassphrase)

            // reset the passphrase to empty value
            enteredWords = Array(repeating: "", count: wordCount)
        } catch {
            self.error = ContinueSessionError.failedToUnlock
            state = .enter
            return
        }
    }
}
