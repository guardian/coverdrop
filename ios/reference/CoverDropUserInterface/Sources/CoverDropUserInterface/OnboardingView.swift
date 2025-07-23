import CoverDropCore
import SwiftUI

struct OnboardingView: View {
    @StateObject var viewModel = OnboardingViewModel()
    @Binding var navPath: NavigationPath

    var body: some View {
        HeaderView(type: .onboarding, dismissAction: {
            if !navPath.isEmpty {
                navPath.removeLast()
            }
        }) {
            VStack(alignment: .leading, spacing: 0) {
                VStack(alignment: .leading, spacing: 0) {
                    TabView(selection: $viewModel.currentStep) {
                        ForEach(OnboardingSteps.allCases) { step in
                            onboardingStep(step: step)
                        }
                    }
                    .tabViewStyle(.page)
                    .indexViewStyle(.page(backgroundDisplayMode: .interactive))
                }
                .padding([.top, .leading, .trailing, .bottom], Padding.large)
                .foregroundColor(Color.white)

                Button(viewModel.currentStep.buttonText) {
                    if viewModel.isFinalStep() {
                        navPath.append(Destination.newPassphrase)
                    } else {
                        advanceToNextTab()
                    }
                }
                .buttonStyle(PrimaryButtonStyle(isDisabled: false))
                .padding([.bottom, .leading, .trailing], Padding.large)
            }
        }
        .onAppear { viewModel.reset() }
        .navigationBarHidden(true)
    }

    func advanceToNextTab() {
        withAnimation {
            viewModel.advanceToNextTab()
        }
    }

    func onboardingStep(step: OnboardingSteps) -> some View {
        // As we show our onboarding steps in a Tab View, the navigation dots cannot
        // extend outside the view port, so we put the contents of the tab in a scroll view
        // so the content can still be viewed at larger text sizes
        ScrollView(.vertical, showsIndicators: false) {
            VStack(alignment: .center, spacing: 0) {
                // Spacers do not work in scroll views, so we add a frame to give it min and max values
                Spacer().frame(minHeight: 30, maxHeight: 200)

                Group {
                    step.image
                        .aspectRatio(contentMode: .fit)
                        .frame(width: 200, height: 200)
                        .padding([.bottom], Padding.large)
                }

                // Spacers do not work in scroll views, so we add a frame to give it min and max values
                Spacer().frame(minHeight: 30, maxHeight: 200)

                Text("How this works")
                    .textStyle(TitleStyle())

                Text(step.rawValue)
                    .textStyle(GuardianHeaderTextStyle())
                    .foregroundColor(Color.OnboardingView.textForegroundColor)
                    .padding(.bottom, Padding.medium)

                Text(step.description)
                    .textStyle(BodyStyle(textAlignment: .center))
                    .padding([.top, .bottom], Padding.small)
                    .padding([.leading, .trailing], Padding.medium)

            }.foregroundColor(Color.white)
            // This makes sure the page content never overlaps the navigation dots
        }.padding(.bottom, 50)
    }
}

class OnboardingViewModel: ObservableObject {
    @Published var currentStep: OnboardingSteps = .sendMessage

    func reset() {
        currentStep = .sendMessage
    }

    func advanceToNextTab() {
        switch currentStep {
        case .sendMessage:
            currentStep = .checkResponse
        case .checkResponse:
            currentStep = .getPassphrase
        case .getPassphrase:
            break
        }
    }

    func isFinalStep() -> Bool {
        currentStep == .getPassphrase
    }
}

enum OnboardingSteps: String, CustomStringConvertible, CaseIterable, Identifiable {
    var id: Self { self }

    case sendMessage = "Send a message"
    case checkResponse = "Check back for a response"
    case getPassphrase = "Memorise your passphrase"

    var description: String {
        switch self {
        case .sendMessage:
            return """
            Your message will be sent anonymously and securely to a journalist or a team. \
            It will be encrypted and indistinguishable from normal network traffic.
            """
        case .checkResponse:
            return """
            Our journalists check for new messages regularly. \
            If they wish to take your story further they will respond here.
            """
        case .getPassphrase:
            return """
            Shortly you will be given a unique passphrase. \
            Use this to access your conversation. \
            Please memorise it or record it somewhere secure.
            """
        }
    }

    var image: Image {
        switch self {
        case .sendMessage:
            return Image(.sendAMessageIcon)
        case .checkResponse:
            return Image(.checkBackForResponseIcon)
        case .getPassphrase:
            return Image(.memorisePassphraseIcon)
        }
    }

    var buttonText: String {
        switch self {
        case .sendMessage:
            return "Continue"
        case .checkResponse:
            return "Continue"
        case .getPassphrase:
            return "Set up my passphrase"
        }
    }
}

#Preview {
    OnboardingView(navPath: Binding.constant(NavigationPath())).previewFonts()
        .environment(CoverDropUserInterfaceConfiguration(showAboutScreenDebugInformation: true, showBetaBanner: true))
}
