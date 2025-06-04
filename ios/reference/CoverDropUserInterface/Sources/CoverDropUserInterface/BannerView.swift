import CoverDropCore
import Foundation
import SwiftUI

struct BetaBannerView: View {
    var showBetaBannerAlert: Binding<Bool>

    init(showBetaBannerAlert: Binding<Bool>) {
        self.showBetaBannerAlert = showBetaBannerAlert
    }

    var body: some View {
        BannerView(
            action: { showBetaBannerAlert.wrappedValue = true },
            imagePath: "exclamationmark.triangle.fill",
            backgroundColor: Color.BetaBannerView.backgroundColor,
            textColor: Color.BetaBannerView.textForegroundColor,
            titleText: "We're testing a new feature",
            subtitleText: "Messages may not all be read"
        )
    }
}

struct PasswordBannerView: View {
    var action: @MainActor () -> Void

    var body: some View {
        BannerView(action: action,
                   imagePath: "info.circle.fill",
                   backgroundColor: Color.PasswordBannerView.backgroundColor,
                   textColor: Color.PasswordBannerView.textForegroundColor,
                   titleText: "Keeping passphrases safe",
                   subtitleText: "Learn more")
    }
}

struct CraftMessageBannerView: View {
    var action: @MainActor () -> Void

    var body: some View {
        BannerView(action: action,
                   imagePath: "info.circle.fill",
                   backgroundColor: Color.PasswordBannerView.backgroundColor,
                   textColor: Color.PasswordBannerView.textForegroundColor,
                   titleText: "Compose your first message",
                   subtitleText: "Learn more")
    }
}

struct BannerView: View {
    var action: @MainActor () -> Void
    var imagePath: String
    var backgroundColor: Color
    var textColor: Color
    var titleText: String
    var subtitleText: String

    var body: some View {
        VStack(alignment: .trailing, spacing: 0) {
            Button(action: self.action) {
                HStack(alignment: .top) {
                    Image(systemName: imagePath)
                        .foregroundColor(textColor)
                        .offset(CGSize(width: 0, height: -3))

                    VStack(alignment: .leading) {
                        Text(titleText)
                            .textStyle(ErrorBoxTitleTextStyle())
                            .foregroundColor(textColor)

                        Text(subtitleText).textStyle(BodyStyle())
                            .padding([.trailing], Padding.small)
                            .foregroundColor(textColor)
                    }

                    Spacer()

                    Image(systemName: "chevron.forward")
                        .resizable()
                        .frame(width: 10, height: 15)
                        .padding(Padding.large)
                        .foregroundColor(textColor)
                }.padding([.top], Padding.medium)
                    .padding([.bottom], Padding.small)
                    .padding([.leading], Padding.xLarge)
                    .contentShape(Rectangle())
            }
            .buttonStyle(.plain)
            .accessibilityIdentifier("Dismiss banner")
        }
        .background(backgroundColor)
    }
}

#Preview {
    Group {
        BetaBannerView(showBetaBannerAlert: .constant(false))
        PasswordBannerView(action: {})
    }.previewFonts()
        .environment(CoverDropUserInterfaceConfiguration(showAboutScreenDebugInformation: true, showBetaBanner: true))
}
