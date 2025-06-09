import Combine
import CoverDropCore
import Foundation
import SwiftUI

enum NewSessionError: Error {
    case wrongPassphrase
    case missingWords
    case misspeltWords
    case failedToCreateStorage
}

extension NewSessionError: LocalizedError {
    public var errorDescription: String? {
        switch self {
        case .wrongPassphrase:
            return "The passphrase you entered does not match the generated one from the previous screen."
        case .missingWords:
            return "Please fill in all passphrase words."
        case .misspeltWords:
            return "The passphrase cannot be right because it contains words that are not in the word list."
        case .failedToCreateStorage:
            return """
            Failed to create a message vault. Please exit Secure Messaging \
            and try again later—or try using a different phone.
            """
        }
    }
}

public enum PasswordField: Int, CaseIterable {
    case password1, password2, password3
}

struct UserNewSessionView: View {
    @Binding var navPath: NavigationPath
    @State var isPasswordHelpOpen: Bool = false
    var passphraseWordCount: Int
    @ObservedObject var viewModel: UserNewSessionViewModel
    @FocusState private var focusedField: PasswordField?

    var body: some View {
        HeaderView(type: .newPassphrase, dismissAction: {
            switch viewModel.state {
            case .generating, .remember:
                navPath.isEmpty ? () : navPath.removeLast()
            case .confirm, .creating, .finished:
                viewModel.goBackToRemember()
            }
        }) {
            PasswordBannerView(action: {
                isPasswordHelpOpen = true
                navPath.append(Destination.help(contentVariant: .keepingPassphraseSafe))
            }).transition(.move(edge: .top))

            VStack(alignment: .leading) {
                switch viewModel.state {
                case .generating:
                    generatingPassphraseView()
                case let .remember(_, passphraseWords, visible):
                    rememberPassphraseView(
                        viewModel: viewModel,
                        passphraseWords: passphraseWords,
                        visible: visible
                    )
                case .confirm:
                    confirmPassphraseView(viewModel: viewModel)
                case .creating:
                    creatingStorageView()
                case .finished:
                    EmptyView()
                }
            }
            .padding(Padding.large)
            .foregroundColor(Color.StartCoverDropSessionView.foregroundColor)
            if case .remember = viewModel.state {
                tertiaryButton(
                    action: {
                        navPath.isEmpty ? () : navPath.removeLast() // remove this screen
                        navPath.isEmpty ? () : navPath.removeLast() // remove onboarding screen
                        navPath.append(Destination.login)
                    },
                    text: "I already have a passphrase"
                ).ignoresSafeArea(.keyboard, edges: .bottom)
            }
        }.navigationBarHidden(true)
            .onAppear {
                Task {
                    viewModel.initializeWithNewPassphrase(passphraseWordCount: passphraseWordCount)
                }
            }
            .onTapGesture {
                focusedField = nil
            }
    }

    func generatingPassphraseView() -> some View {
        VStack {
            Spacer()
            ProgressView()
                .progressViewStyle(CircularProgressViewStyle())
                .tint(.white)
                .padding(Padding.large)
            Text("Generating passphrase...").textStyle(BodyStyle())
            Spacer()
        }
    }

    func creatingStorageView() -> some View {
        VStack {
            Spacer()
            ProgressView()
                .progressViewStyle(CircularProgressViewStyle())
                .tint(.white)
                .padding(Padding.large)
            Text("Creating storage...").textStyle(BodyStyle())
            Spacer()
        }
    }

    func rememberPassphraseView(
        viewModel: UserNewSessionViewModel,
        passphraseWords: [String],
        visible: Bool
    ) -> some View {
        VStack(alignment: .leading) {
            Text("Remember Passphrase").textStyle(TitleStyle())
            Text(getRememberPassphraseInfoText())
                .textStyle(BodyStyle())
                .padding(.bottom, Padding.small)
                .fixedSize(horizontal: false, vertical: true)
            VStack {
                passphraseWordListView(passphraseWords: passphraseWords, visible: visible)
            }.padding(.bottom, Padding.small)

            hideShowButton(
                visible: visible,
                viewModel: viewModel,
                textShowAll: "Reveal passphrase",
                textHideAll: "Hide passphrase"
            )

            Spacer()

            if visible {
                Button("I have remembered my passphrase") {
                    viewModel.advanceToEnter()
                }.buttonStyle(PrimaryButtonStyle(isDisabled: false))
            } else {
                Button("\(Image(systemName: "eye.fill")) Reveal passphrase") {
                    viewModel.showAll()
                }.buttonStyle(PrimaryButtonStyle(isDisabled: false))
                    .accessibilityLabel("Reveal passphrase to continue")
            }
        }
    }

    func confirmPassphraseView(viewModel: UserNewSessionViewModel) -> some View {
        VStack(alignment: .leading) {
            Text("Enter passphrase")
                .textStyle(TitleStyle(bottomPadding: Padding.medium))
            Text("Enter your passphrase to unlock your secure vault and send your first message.")
                .textStyle(BodyStyle(bottomPadding: Padding.large))
                .fixedSize(horizontal: false, vertical: true)

            if viewModel.error != nil {
                Text(viewModel.error?.localizedDescription ?? "")
                    .textStyle(FormErrorTextStyle())
                    .padding(.bottom, Padding.large)
                    .fixedSize(horizontal: false, vertical: true)
            }

            PassphraseFormView(
                wordCount: passphraseWordCount,
                words: $viewModel.enteredWords,
                wordVisible: $viewModel.wordVisible,
                wordInvalid: viewModel.invalidWords,
                passwordFieldFocus: $focusedField
            )

            hideShowButton(
                visible: viewModel.wordVisible.allSatisfy { $0 },
                viewModel: viewModel
            )

            Spacer()

            Button("Confirm passphrase") {
                Task {
                    await viewModel.createNewStorage()
                }
            }.buttonStyle(PrimaryButtonStyle(isDisabled: false))
        }
    }

    private func passphraseWordListView(passphraseWords: [String], visible: Bool) -> some View {
        VStack(spacing: Padding.small) {
            ForEach(Array(passphraseWords.enumerated()), id: \.element) { id, word in
                if visible {
                    Text(word)
                        .textStyle(PassphraseTextStyle())
                        .accessibilityIdentifier("Word \(id + 1)")
                } else {
                    Text("••••••").textStyle(PassphraseTextStyle())
                }

            }.frame(maxWidth: .infinity)
        }.background(Color.UserNewSessionView.wordListBackgroundColor)
            .frame(maxWidth: .infinity)
    }

    private func hideShowButton(
        visible: Bool,
        viewModel: UserNewSessionViewModel,
        textShowAll: String = "Reveal passphrase",
        textHideAll: String = "Hide passphrase"
    ) -> some View {
        HStack {
            Spacer()
            if visible {
                Button("\(Image(systemName: "eye.slash.fill")) \(textHideAll)") {
                    viewModel.hideAll()
                }.buttonStyle(InlineButtonStyle())
                    .accessibilityLabel("\(textHideAll) fields")
            } else {
                Button("\(Image(systemName: "eye.fill")) \(textShowAll)") {
                    viewModel.showAll()
                }.buttonStyle(InlineButtonStyle())
                    .accessibilityLabel("\(textShowAll) fields")
            }
        }
    }

    private func getRememberPassphraseInfoText() -> AttributedString {
        let as1 = AttributedString("You will always need this passphrase to access your conversation.")
        var as2 = AttributedString("Make sure you memorise it or write it down somewhere safe.")
        as2.foregroundColor = Color.HelpExample.highlightColor
        as2.font = .body.weight(.semibold)
        let as3 = AttributedString("Your passphrase must be entered in the correct order with the correct spelling.")

        return as1 + " " + as2 + " " + as3
    }
}

class UserNewSessionViewModel: ObservableObject {
    enum State: Equatable {
        case generating
        case remember(passphrase: ValidPassword, passphraseWords: [String], visible: Bool)
        case confirm(passphrase: ValidPassword)
        case creating(passphrase: ValidPassword)
        case finished
    }

    var lib: CoverDropLibrary
    var validPrefixes = Set<String>()

    @MainActor @Published var state: State = .generating
    @MainActor @Published var error: Error?

    // We keep these separate to so that they can survive navigating back-and-forth and allow easier
    // binding from the TextFields
    @Published var enteredWords: [String] = []
    @Published var wordVisible: [Bool] = []

    var invalidWords: [Bool] {
        // return `true` for each non-empty word that is not a valid prefix
        enteredWords.map { word in !word.isEmpty && !validPrefixes.contains(word) }
    }

    public init(lib: CoverDropLibrary) {
        self.lib = lib
    }

    @MainActor func initializeWithNewPassphrase(passphraseWordCount: Int) {
        if case .generating = state {
            validPrefixes = PasswordGenerator.shared.generatePrefixes()
            let passphrase = PasswordGenerator.shared.generate(wordCount: passphraseWordCount)
            let passphraseWords = passphrase.words

            state = .remember(
                passphrase: passphrase,
                passphraseWords: passphraseWords,
                visible: false
            )
        }
    }

    @MainActor func showAll() {
        if case let .remember(passphrase, passphraseWords, _) = state {
            state = .remember(passphrase: passphrase, passphraseWords: passphraseWords, visible: true)
        }
        if case .confirm = state {
            wordVisible = Array(repeating: true, count: enteredWords.count)
        }
    }

    @MainActor func hideAll() {
        if case let .remember(passphrase, passphraseWords, _) = state {
            state = .remember(passphrase: passphrase, passphraseWords: passphraseWords, visible: false)
        }
        if case .confirm = state {
            wordVisible = Array(repeating: false, count: enteredWords.count)
        }
    }

    @MainActor func advanceToEnter() {
        guard case let .remember(passphrase, passphraseWords, _) = state else {
            return
        }

        // If the user navigates to the enter screen for the first time, we need to initialise the enteredWords array.
        // However, we don't want to redo this in case they briefly navigated back once to have another look at the
        // remember screen.
        if enteredWords.isEmpty {
            enteredWords = Array(repeating: "", count: passphraseWords.count)
            wordVisible = Array(repeating: true, count: passphraseWords.count)
        }

        state = .confirm(passphrase: passphrase)
    }

    @MainActor func goBackToRemember() {
        guard case let .confirm(passphrase) = state else {
            return
        }
        state = .remember(
            passphrase: passphrase,
            passphraseWords: passphrase.words,
            visible: false
        )
    }

    func createNewStorage() async {
        guard case let .confirm(passphrase) = await state else {
            return
        }

        if enteredWords.contains(where: { $0.isEmpty }) {
            await MainActor.run {
                error = NewSessionError.missingWords
            }
            return
        }

        let enteredPassphrase = enteredWords.joined(separator: " ")

        guard let validatedPassphrase = try? PasswordGenerator.checkValid(passwordInput: enteredPassphrase) else {
            await MainActor.run {
                error = NewSessionError.misspeltWords
            }
            return
        }

        if validatedPassphrase != passphrase {
            await MainActor.run {
                error = NewSessionError.wrongPassphrase
            }
            return
        } else {
            await MainActor.run {
                error = nil
            }
        }

        await MainActor.run { state = .creating(passphrase: passphrase) }

        do {
            _ = try await lib.secretDataRepository.createOrReset(passphrase: passphrase)
            try? await lib.secretDataRepository.unlock(passphrase: passphrase)
        } catch {
            await MainActor.run {
                self.error = NewSessionError.failedToCreateStorage
            }
            return
        }
    }
}

#Preview {
    UserNewSessionView(
        navPath: Binding.constant(NavigationPath()), passphraseWordCount: 3,
        // swiftlint:disable:next force_try
        viewModel: UserNewSessionViewModel(lib: try! IntegrationTestScenarioContext(
            scenario: IntegrationTestScenario.minimal
        ).getLibraryWithVerifiedKeys())
    ).previewFonts()
        .environment(CoverDropUserInterfaceConfiguration(showAboutScreenDebugInformation: true, showBetaBanner: true))
}
