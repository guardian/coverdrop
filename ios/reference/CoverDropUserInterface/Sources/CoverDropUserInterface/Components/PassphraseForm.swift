import Foundation
import SwiftUI

struct PassphraseFormView: View {
    var wordCount: Int
    @Binding var words: [String]
    @Binding var wordVisible: [Bool]
    var wordInvalid: [Bool]
    var passwordFieldFocus: FocusState<PasswordField?>.Binding

    var body: some View {
        VStack(spacing: 0) {
            ForEach(0 ... wordCount - 1, id: \.self) { id in
                var passwordFieldFocusValue: PasswordField {
                    if id == 0 { .password1 } else if id == 1 { .password2 } else { .password3 }
                }
                if wordVisible[id] {
                    clearTextPassphaseInput(id: id, passwordFieldFocusValue: passwordFieldFocusValue)
                } else {
                    secureTextPassphaseInput(id: id, passwordFieldFocusValue: passwordFieldFocusValue)
                }
            }
        }.padding(.bottom, Padding.small)
        .onChange(of: $words.wrappedValue) { _, newValue in
            // This fixes an issue with autocorrect that always adds a space at the end of a corrected word
            $words.wrappedValue = newValue.map { value in
                value.trimmingCharacters(in: .whitespaces)
            }
        }
        .onSubmit {
            // This focuses on the next text field when the user presses the enter key
            switch passwordFieldFocus.wrappedValue {
            case .password1:
                passwordFieldFocus.wrappedValue = .password2
            case .password2:
                passwordFieldFocus.wrappedValue = .password3
            default:
                passwordFieldFocus.wrappedValue = nil
            }
        }
    }

    func clearTextPassphaseInput(id: Int, passwordFieldFocusValue: PasswordField) -> some View {
        ZStack(alignment: .topTrailing) {
            Rectangle()
                .fill(Color.clear)
                .frame(height: 60) // Larger than the text field's height
                .contentShape(Rectangle())
                .onTapGesture {
                    passwordFieldFocus.wrappedValue = passwordFieldFocusValue
                }
            TextField("", text: $words[id])
                .textFieldStyle(PassphraseFieldStyle(isError: wordInvalid[id]))
                .accessibilityIdentifier("Passphrase Word \(id + 1)")
                .focused(
                    passwordFieldFocus,
                    equals: passwordFieldFocusValue
                )
                .submitLabel(id == wordCount - 1 ? .done : .next)
                .autocorrectionDisabled(true)
                .textInputAutocapitalization(.never)
                .disableAutocorrection(true) // extra safety
                .keyboardType(.asciiCapable) // avoids predictive bar triggers which cause hangs

            Button(action: {
                $wordVisible.wrappedValue[id] = false
                passwordFieldFocus.wrappedValue = passwordFieldFocusValue
            }, label: {
                Image(systemName: "eye.slash.fill")
            }).accessibilityIdentifier("show \(id + 1)")
                .padding(Padding.medium)
                .frame(height: 52)
                .background(Color.clear)
                .contentShape(Rectangle())
        }
    }

    func secureTextPassphaseInput(id: Int, passwordFieldFocusValue: PasswordField) -> some View {
        ZStack(alignment: .topTrailing) {
            Rectangle()
                .fill(Color.clear)
                .frame(height: 60) // Larger than the text field's height
                .contentShape(Rectangle())
                .onTapGesture {
                    passwordFieldFocus.wrappedValue = passwordFieldFocusValue
                }
            SecureField("", text: $words[id])
                .textFieldStyle(PassphraseFieldStyle(isError: wordInvalid[id]))
                .textContentType(.password)
                .accessibilityIdentifier("Passphrase Word \(id + 1)")
                .focused(passwordFieldFocus, equals: passwordFieldFocusValue)
                .submitLabel(id == wordCount - 1 ? .done : .next)
                .keyboardType(.asciiCapable)
            Button(action: {
                $wordVisible.wrappedValue[id] = true
                passwordFieldFocus.wrappedValue = passwordFieldFocusValue
            }, label: {
                Image(systemName: "eye.fill")
            }).accessibilityIdentifier("hide \(id + 1)")
                .padding(Padding.medium)
                .frame(height: 52)
                .background(Color.clear)
                .contentShape(Rectangle())
        }
    }
}
