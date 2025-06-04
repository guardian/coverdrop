import CoverDropCore
import Foundation
import SwiftUI

public struct StartCoverDropSessionView: View {
    @ObservedObject var publicDataRepository: PublicDataRepository
    @Binding var navPath: NavigationPath

    /// Controls whether the alert is shown before starting a new conversation
    @State private var showingNewMessageAlert = false

    public var body: some View {
        HeaderView(type: .home) {
            VStack(alignment: .leading) {
                titleText.textStyle(LargeTitleStyle()).font(Font.headline.leading(.loose))

                Text(
                    """
                    The Guardian Secure Messaging service was developed to allow you to share stories with us \
                    securely and privately using strong encryption.
                    """
                )
                .textStyle(BodyStyle())
                .padding(.top, Padding.medium)
                .padding(.bottom, Padding.small)

                Text("It is designed to prevent others from even knowing you are in communication with us.")
                    .textStyle(BodyStyle())
                    .padding(.top, Padding.small)
                    .padding(.bottom, Padding.medium)

                Button(action: {
                    navPath.append(Destination.about)
                }) {
                    Text("About Secure Messaging")
                        .fontWeight(.bold)
                    Image(systemName: "chevron.forward")
                        .resizable()
                        .fontWeight(.black)
                        .frame(width: 7, height: 11)
                        .foregroundColor(Color.ChevronButtonList.chevronColor)
                        .padding([.trailing], Padding.small)
                }

                Spacer()

                if let coverDropServiceStatus = publicDataRepository.coverDropServiceStatus,
                   coverDropServiceStatus.isAvailable == false {
                    VStack {
                        Text(
                            """
                            The Secure Messaging feature is currently not available.
                            Please try again later. Below we show technical information that might be helpful.
                            """
                        )
                        Text(coverDropServiceStatus.description).textStyle(MonoSpacedStyle())
                    }

                } else {
                    Button("Get started") {
                        showingNewMessageAlert = true
                    }
                    .disabled(!publicDataRepository.areKeysAvailable)
                    .buttonStyle(PrimaryButtonStyle(isDisabled: !publicDataRepository.areKeysAvailable))
                    .alert("Set up your secure message vault", isPresented: $showingNewMessageAlert, actions: {
                        Button("Yes, start conversation") {
                            showingNewMessageAlert = false
                            navPath.append(Destination.onboarding)
                        }
                        Button("No", role: .cancel) {}
                    }, message: {
                        Text(
                            """
                            Starting a new conversation will remove any prior messages from your vault, \
                            if they existed.
                            Do you want to continue?
                            """
                        )
                    })

                    Button("Check your message vault") {
                        navPath.append(Destination.login)
                    }.buttonStyle(SecondaryButtonStyle(isDisabled: false))
                }

            }.padding(Padding.large)
                .foregroundColor(Color.StartCoverDropSessionView.foregroundColor)
        }
    }

    var titleText: Text {
        Text("Send us a message ").foregroundColor(Color.StartCoverDropSessionView.firstLineTextForegroundColor)
            + Text("securely").foregroundColor(Color.StartCoverDropSessionView.secondLineTextForegroundColor)
            + Text(" and ").foregroundColor(Color.StartCoverDropSessionView.firstLineTextForegroundColor)
            + Text("privately").foregroundColor(Color.StartCoverDropSessionView.secondLineTextForegroundColor)
    }
}

// struct StartCoverDropSessionView_Previews: PreviewProvider {
//    @MainActor struct Container: View {
//        let setup: () = setupView()
//        var setupRepo: () = PublicDataRepository.setup(StaticConfig.devConfig)
//        let publicDataRepository: PublicDataRepository = PublicDataRepository.shared
//        let viewModel = StartCoverDropSessionViewModel(publicDataRepository: PublicDataRepository.shared)
//
//        @MainActor var body: some View {
//            let _: () = setAvailabe()
//            PreviewWrapper(StartCoverDropSessionView(publicDataRepository: publicDataRepository,
// viewModel: viewModel))
//            let _: () = setUnavailabe()
//            PreviewWrapper(StartCoverDropSessionView(publicDataRepository: publicDataRepository,
// viewModel: viewModel))
//        }
//    }
//
//    public static func setAvailabe() {
//        PublicDataRepository.shared.coverDropServiceStatus = StatusData(
//            status: .available,
//            description: "Service is available",
//            timestamp: RFC3339DateTimeString(date: DateFunction.currentTime()),
//            isAvailable: true
//        )
//    }
//
//    public static func setUnavailabe() {
//        PublicDataRepository.shared.coverDropServiceStatus = StatusData(
//            status: .noInformation,
//            description: "Service is unavailable",
//            timestamp: RFC3339DateTimeString(date: DateFunction.currentTime()),
//            isAvailable: false
//        )
//    }
//
//    public static func setupView() {
//        PublicDataRepository.setup(StaticConfig.devConfig)
//        PublicDataRepository.shared.areKeysAvailable = true
//        CoverDropServices.shared.isReady = true
//        PublicDataRepository.shared.coverDropServiceStatus = StatusData(
//            status: .available,
//            description: "",
//            timestamp: RFC3339DateTimeString(date: DateFunction.currentTime()),
//            isAvailable: true
//        )
//    }
//
//    static var previews: some View {
//        Container()
//    }
// }
