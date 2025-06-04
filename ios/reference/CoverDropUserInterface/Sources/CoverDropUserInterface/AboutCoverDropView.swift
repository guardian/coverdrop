import CoverDropCore
import Foundation
import SwiftUI

struct ChevronButtonData {
    var text: String
    var target: HelpScreenContent
}

struct ChevronButtonList: View {
    @Binding var navPath: NavigationPath
    @State var buttonData = [ChevronButtonData]()

    func navigateToHelp(contentVariant: HelpScreenContent) {
        navPath.append(Destination.help(contentVariant: contentVariant))
    }

    var body: some View {
        VStack {
            VStack(alignment: .leading, spacing: 0) {
                ForEach(buttonData.indices, id: \.self) { index in
                    let data = buttonData[index]
                    Button(action: { navigateToHelp(contentVariant: data.target) }) {
                        HStack {
                            Text(data.text)
                                .textStyle(BodyStyle())
                                .fontWeight(.semibold)
                                .padding([.top], 8)
                                .padding([.bottom], 4)
                            Spacer()
                            Image(systemName: "chevron.forward")
                                .resizable()
                                .fontWeight(.semibold)
                                .frame(width: 7, height: 11)
                                .foregroundColor(Color.ChevronButtonList.chevronColor)
                                .padding([.trailing], Padding.medium)
                        }
                    }
                    if index < buttonData.count - 1 {
                        Rectangle()
                            .fill(Color.ChevronButtonList.dividerColor)
                            .frame(height: 1)
                            .padding(.vertical, Padding.small)
                    }
                }
            }.padding(.leading, Padding.large)
                .padding(.vertical, Padding.small)
        }.background(Color.ChevronButtonList.backgroundColor)
            .cornerRadius(CornerRadius.medium)
    }
}

struct AboutCoverDropView: View {
    @Binding var navPath: NavigationPath
    @Environment(CoverDropUserInterfaceConfiguration.self) var uiConfig
    @ObservedObject var viewModel: AboutCoverDropViewModel

    func navigateToHelp(contentVariant: HelpScreenContent) {
        navPath.append(Destination.help(contentVariant: contentVariant))
    }

    var body: some View {
        HeaderView(type: .about, dismissAction: {
            if !navPath.isEmpty {
                navPath.removeLast()
            }
        }) {
            ScrollView {
                VStack(alignment: .leading) {
                    Text("About Secure Messaging")
                        .textStyle(LargeTitleStyle())
                        .font(Font.headline.leading(.loose))

                    Text("What this is for")
                        .textStyle(GuardianHeaderTextStyle())
                        .padding([.top], Padding.large)
                        .padding([.bottom], Padding.small)
                    ChevronButtonList(navPath: $navPath, buttonData: [
                        ChevronButtonData(
                            text: "Why we made Secure Messaging",
                            target: .whyWeMadeSecureMessaging
                        ),
                        ChevronButtonData(
                            text: "How Secure Messaging works",
                            target: .howSecureMessagingWorks
                        ),
                        ChevronButtonData(
                            text: "FAQs",
                            target: .faq
                        ),
                        ChevronButtonData(
                            text: "Privacy policy",
                            target: .privacyPolicy
                        )
                    ])

                    Text("Getting started")
                        .textStyle(GuardianHeaderTextStyle())
                        .padding([.top], Padding.large)
                        .padding([.bottom], Padding.small)
                    ChevronButtonList(navPath: $navPath, buttonData: [
                        ChevronButtonData(
                            text: "Compose your first message",
                            target: .craftMessage
                        ),
                        ChevronButtonData(
                            text: "Keeping passphrases safe",
                            target: .keepingPassphraseSafe
                        )
                    ])

                    Text("As the conversation progresses")
                        .textStyle(GuardianHeaderTextStyle())
                        .padding([.top], Padding.large)
                        .padding([.bottom], Padding.small)
                    ChevronButtonList(navPath: $navPath, buttonData: [
                        ChevronButtonData(
                            text: "What to expect as a reply",
                            target: .replyExpectations
                        ),
                        ChevronButtonData(
                            text: "Source protection",
                            target: .sourceProtection
                        )
                    ])

                    if uiConfig.showAboutScreenDebugInformation {
                        Text("Technical information")
                            .textStyle(GuardianHeaderTextStyle())
                            .padding([.top], Padding.large)
                            .padding([.bottom], Padding.small)
                        Text("The details below help our engineers to develop and test this feature.")
                            .padding([.bottom], Padding.small)
                        DebugContextView(state: viewModel.state)
                            .onAppear {
                                Task {
                                    await viewModel.loadDebugContextInformation()
                                }
                            }
                    }
                }.padding(Padding.large)
                    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
            }
            Spacer()
        }.foregroundColor(Color.StartCoverDropSessionView.foregroundColor)
            .navigationBarHidden(true)
    }
}

struct DebugContextView: View {
    var state: AboutCoverDropViewModel.State

    var body: some View {
        switch state {
        case .loading:
            ProgressView()
                .progressViewStyle(CircularProgressViewStyle())
                .tint(.white)
        case let .ready(debugContext):
            Text(debugContext.description)
                .font(.system(size: 14, design: .monospaced))
        case .error:
            Text("Error while loading debug information.")
        }
    }
}

class AboutCoverDropViewModel: ObservableObject {
    enum State {
        case loading
        case ready(debugContext: DebugContext)
        case error
    }

    var lib: CoverDropLibrary

    @MainActor @Published var state: State = .loading

    public init(lib: CoverDropLibrary) {
        self.lib = lib
    }

    func loadDebugContextInformation() async {
        do {
            let debugContext = try await lib.publicDataRepository.getDebugContext()
            await MainActor.run {
                state = .ready(debugContext: debugContext)
            }
        } catch {
            Debug.println("Failed loading debug context: \(error)")
            await MainActor.run {
                state = .error
            }
        }
    }
}
