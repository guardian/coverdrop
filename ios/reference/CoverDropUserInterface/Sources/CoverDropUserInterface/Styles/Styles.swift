import Foundation
import GuardianFonts
import SwiftUI

enum Padding {
    static let xLarge: CGFloat = 21
    static let large: CGFloat = 16
    static let medium: CGFloat = 12
    static let small: CGFloat = 5
    static let xSmall: CGFloat = 2
}

enum FontSize {
    static let largeTitle: CGFloat = 32
    static let title: CGFloat = 27
    static let smallHeadlineTitle: CGFloat = 22
    static let xSmallHeadlineTitle: CGFloat = 15
    static let textField: CGFloat = 17
    static let buttonText: CGFloat = 17
    static let segmentedControlText: CGFloat = 12
    static let bodyText: CGFloat = 17
    static let listText: CGFloat = 16
    static let xSmallButtonText: CGFloat = 15
    static let inlineButtonText: CGFloat = 14
    static let listDetailText: CGFloat = 13
    static let messageMetadataText: CGFloat = 12
}

enum CornerRadius {
    static let large: CGFloat = 30
    static let medium: CGFloat = 6
    static let small: CGFloat = 4
}

struct PassphraseFieldStyle: TextFieldStyle {
    var isError: Bool = false
    // swiftlint:disable:next identifier_name
    func _body(configuration: TextField<Self._Label>) -> some View {
        configuration
            .frame(height: 20)
            .padding(Padding.large)
            .monospaced()
            .background(
                RoundedRectangle(cornerRadius: CornerRadius.small).stroke(
                    isError ? Color.PassphraseFieldStyle.errorColor : Color.PassphraseFieldStyle.strokeColor,
                    lineWidth: 1
                )
            )
            .background(Color.PassphraseFieldStyle.backgroundColor)
    }
}

func customDivider() -> some View {
    Rectangle().fill(Color.CustomDivider.backgroundColor)
        .padding(0)
        .frame(height: 1)
}

func tertiaryButton(action: @escaping () -> Void, text: String) -> some View {
    Button(action: action, label: {
        HStack {
            Text(text)
            Spacer()
            Image(systemName: "chevron.forward")
                .resizable()
                .scaledToFit()
                .frame(width: 18, height: 25)
        }
    }).buttonStyle(TeriaryButtonStyle())
}

struct LargeTitleStyle: ViewModifier {
    func body(content: Content) -> some View {
        content
            .font(.headlineBold, size: FontSize.largeTitle)
            .padding(.bottom, Padding.medium)
    }
}

struct TitleStyle: ViewModifier {
    var bottomPadding: CGFloat = Padding.medium
    func body(content: Content) -> some View {
        content
            .font(.headlineBold, size: FontSize.title, lineHeight: 39)
            .padding(.bottom, bottomPadding)
            .fixedSize(horizontal: false, vertical: false)
    }
}

struct BodyStyle: ViewModifier {
    var textAlignment: TextAlignment = .leading
    var bottomPadding: CGFloat = Padding.small
    func body(content: Content) -> some View {
        content
            //    The text alignment parameter was an addition we made in the local font library that is not
            //    available in the remote one, I've opened a PR to add it.
            //            .font(.textSansRegular,
            //                  size: FontSize.bodyText,
            //                  lineHeight: 23,
            //                  alignment: textAlignment)
            .font(Font.custom(
                GuardianFontStyle.textSansRegular.fontName,
                size: FontSize.bodyText,
                relativeTo: GuardianFontStyle.textSansRegular.relativeStyle
            )).multilineTextAlignment(textAlignment)
            .padding(.bottom, bottomPadding)
    }
}

struct MessageMetadata: ViewModifier {
    func body(content: Content) -> some View {
        content
            .font(.textSansRegular,
                  size: FontSize.messageMetadataText, lineHeight: 23)
            .padding(.bottom, Padding.medium)
            .foregroundColor(Color.MessageMetadata.foregroundColor)
    }
}

struct ExpiringMessageMetadata: ViewModifier {
    func body(content: Content) -> some View {
        content
            .font(.textSansBold,
                  size: FontSize.messageMetadataText, lineHeight: 23)
            .padding(.bottom, Padding.small)
            .foregroundColor(Color.NewMessageView.messageInformationStrokeColor)
    }
}

struct PendingMessageMetadata: ViewModifier {
    func body(content: Content) -> some View {
        content
            .font(.textSansBold,
                  size: FontSize.messageMetadataText, lineHeight: 23)
            .padding(.bottom, Padding.small)
            .foregroundColor(Color.NewMessageView.messageInformationStrokeColor)
    }
}

struct SentMessageMetadata: ViewModifier {
    func body(content: Content) -> some View {
        content
            .font(.textSansBold,
                  size: FontSize.messageMetadataText, lineHeight: 23)
            .padding(.bottom, Padding.small)
            .foregroundColor(Color.JournalistNewMessageView.tickIconColor)
    }
}

struct MessageTextErrorMessage: LabelStyle {
    func makeBody(configuration: Configuration) -> some View {
        Label(configuration)
            .foregroundColor(Color.NewMessageView.messageToLongErrorColor)
            .font(.textSansBold,
                  size: FontSize.bodyText,
                  lineHeight: 23)
    }
}

struct UserNotificationTextStyle: ViewModifier {
    func body(content: Content) -> some View {
        content
            .font(.textSansMedium,
                  size: FontSize.bodyText,
                  lineHeight: 23)
            .padding(.bottom, Padding.xSmall)
            .padding([.trailing, .leading], Padding.medium)
            .multilineTextAlignment(.center)
    }
}

struct FormLabelTextStyle: ViewModifier {
    func body(content: Content) -> some View {
        content
            .font(.textSansBold,
                  size: FontSize.bodyText,
                  lineHeight: 25)
            .padding(.bottom, Padding.medium)
    }
}

struct FormLabelSubtitleTextStyle: ViewModifier {
    func body(content: Content) -> some View {
        content
            .font(.textSansRegular,
                  size: FontSize.bodyText, lineHeight: 25)
            .padding(.bottom, Padding.medium)
    }
}

struct ErrorBoxTitleTextStyle: ViewModifier {
    func body(content: Content) -> some View {
        content
            .font(.textSansBold,
                  size: FontSize.bodyText,
                  lineHeight: 25)
            .padding(.bottom, Padding.small)
    }
}

struct FormErrorTextStyle: ViewModifier {
    func body(content: Content) -> some View {
        content
            .font(.textSansBold,
                  size: FontSize.bodyText,
                  lineHeight: 25)
            .foregroundColor(Color.UserNewSessionView.errorColor)
            .padding(.bottom, Padding.small)
    }
}

struct InlineButtonTextStyle: ViewModifier {
    func body(content: Content) -> some View {
        content
            .font(.textSansBold,
                  size: FontSize.inlineButtonText,
                  lineHeight: 23)
            .padding([.trailing, .top, .bottom], Padding.small)
    }
}

struct PassphraseTextStyle: ViewModifier {
    func body(content: Content) -> some View {
        return content
            .font(.textSansBold, size: FontSize.textField)
            .monospaced()
            .padding(Padding.large)
            .frame(maxWidth: .infinity, minHeight: 50, alignment: .center)
            .overlay(
                RoundedRectangle(cornerRadius: CornerRadius.small)
                    .stroke(Color.PassphraseTextStyle.strokeColor, style: StrokeStyle(lineWidth: 1, dash: [3]))
            )
    }
}

struct MonoSpacedStyle: ViewModifier {
    func body(content: Content) -> some View {
        return content
            .font(.system(.body, design: .monospaced))
            .padding(Padding.large)
    }
}

struct SelectRecipientTextStyle: ViewModifier {
    func body(content: Content) -> some View {
        return content
            .font(.textSansRegular, size: FontSize.textField)
            .padding(Padding.large)
            .frame(maxWidth: .infinity, alignment: .leading)
            .overlay(
                RoundedRectangle(cornerRadius: CornerRadius.small)
                    .stroke(Color.SelectRecipientTextStyle.strokeColor, style: StrokeStyle(lineWidth: 1))
            )
            .background(Color.SelectRecipientTextStyle.backgroundColor)
    }
}

struct GuardianHeaderTextStyle: ViewModifier {
    func body(content: Content) -> some View {
        return content
            .font(.headlineBold, size: FontSize.smallHeadlineTitle, lineHeight: 30)
            .padding([.top, .bottom], Padding.small)
    }
}

struct GuardianHeadlineXSmallHeaderTextStyle: ViewModifier {
    func body(content: Content) -> some View {
        return content
            .font(.headlineBold, size: FontSize.xSmallHeadlineTitle)
            .padding([.top, .bottom], Padding.small)
    }
}

struct GuardianHeadlineSmallTextStyle: ViewModifier {
    func body(content: Content) -> some View {
        return content
            .font(.headlineBold, size: FontSize.bodyText)
            .padding([.top, .bottom], Padding.small)
    }
}

struct RecipientTextStyle: ViewModifier {
    func body(content: Content) -> some View {
        return content
            .font(.textSansRegular, size: FontSize.textField)
            .padding(Padding.large)
            .frame(maxWidth: .infinity, alignment: .leading)
            .overlay(
                RoundedRectangle(cornerRadius: CornerRadius.small)
                    .stroke(Color.RecipientTextStyle.strokeColor, style: StrokeStyle(lineWidth: 1))
            )
    }
}

struct MessageHelperTextStyle: ViewModifier {
    func body(content: Content) -> some View {
        return content
            .font(.textSansRegular, size: FontSize.textField)
            .multilineTextAlignment(.center)
            .padding([.top, .leading, .trailing], 10)
            .padding([.bottom], 10)
    }
}

struct ListItemTitleTextStyle: ViewModifier {
    func body(content: Content) -> some View {
        return content
            .font(.textSansBold,
                  size: FontSize.listText)
            .foregroundColor(Color.ListItemTitleTextStyle.foregroundColor)
            .padding(.leading, -10)
    }
}

struct ListItemDetailTextStyle: ViewModifier {
    func body(content: Content) -> some View {
        return content
            .font(.textSansRegular,
                  size: FontSize.listDetailText)
            .foregroundColor(Color.ListItemDetailTextStyle.foregroundColor)
            .padding(.leading, -10)
    }
}

struct MessageDetailTextStyle: ViewModifier {
    func body(content: Content) -> some View {
        return content
            .font(.textSansBold,
                  size: 12)
            .foregroundColor(Color.InboxView.lastMessageColor)
            .padding(.top, Padding.small)
    }
}

extension Text {
    func textStyle<Style: ViewModifier>(_ style: Style) -> some View {
        ModifiedContent(content: self, modifier: style)
    }
}

struct FooterButtonStyle: ButtonStyle {
    func makeBody(configuration: Self.Configuration) -> some View {
        return configuration.label
            .frame(maxWidth: .infinity)
            .foregroundColor(Color.FooterButtonStyle.foregroundColor)
            .padding([.bottom], Padding.medium)
            .padding([.top], Padding.xLarge)
            .font(.textSansBold, size: FontSize.buttonText)
    }
}

struct HideButtonStyle: ButtonStyle {
    func makeBody(configuration: Self.Configuration) -> some View {
        return configuration.label
            .foregroundColor(Color.HideButtonStyle.foregroundColor)
            .padding([.bottom], Padding.small)
            .padding([.top], Padding.xLarge)
            .font(.textSansBold, size: FontSize.buttonText)
    }
}

struct PrimaryButtonStyle: ButtonStyle {
    let isDisabled: Bool
    let backgroundColor = Color.PrimaryButtonStyle.backgroundColor
    let foregroundColor = Color.PrimaryButtonStyle.foregroundColor

    func makeBody(configuration: Self.Configuration) -> some View {
        return LargeButtonStyle(
            backgroundColor: backgroundColor,
            foregroundColor: foregroundColor,
            isDisabled: isDisabled,
            stroke: false
        ).makeBody(configuration: configuration)
    }
}

struct SecondaryButtonStyle: ButtonStyle {
    let isDisabled: Bool
    let backgroundColor = Color.SecondaryButtonStyle.backgroundColor
    let foregroundColor = Color.SecondaryButtonStyle.foregroundColor

    func makeBody(configuration: Self.Configuration) -> some View {
        return LargeButtonStyle(
            backgroundColor: backgroundColor,
            foregroundColor: foregroundColor,
            isDisabled: isDisabled,
            stroke: true
        ).makeBody(configuration: configuration)
    }
}

struct TeriaryButtonStyle: ButtonStyle {
    func makeBody(configuration: Self.Configuration) -> some View {
        return VStack {
            customDivider()
            configuration.label
                .frame(maxWidth: .infinity)
                .foregroundColor(Color.TertiaryButtonStyle.foregroundColor)
                .padding([.leading, .trailing], Padding.xLarge)
                .padding([.top], Padding.medium)
                .padding([.bottom], Padding.large)
                .font(.textSansBold, size: FontSize.buttonText)
        }
    }
}

struct InlineButtonStyle: ButtonStyle {
    func makeBody(configuration: Self.Configuration) -> some View {
        configuration.label
            .font(.textSansBold,
                  size: FontSize.inlineButtonText,
                  lineHeight: 23)
            .padding([.trailing, .top, .bottom], Padding.small)
    }
}

struct LargeButtonStyle: ButtonStyle {
    let backgroundColor: Color
    let foregroundColor: Color
    let isDisabled: Bool
    let stroke: Bool

    func makeBody(configuration: Self.Configuration) -> some View {
        let currentForegroundColor = isDisabled || configuration.isPressed ? Color.LargeButtonStyle
            .highlightForegroundColor : foregroundColor
        return configuration.label
            .frame(maxWidth: .infinity, alignment: .center)
            .multilineTextAlignment(.center)
            .padding(Padding.medium)
            .padding([.leading, .trailing], Padding.xLarge)
            .foregroundColor(currentForegroundColor)
            .background(isDisabled || configuration.isPressed ? Color.LargeButtonStyle
                .highlightBackgroundColor : backgroundColor)
            // This is the key part, we are using both an overlay as well as cornerRadius
            .cornerRadius(CornerRadius.large)
            .overlay(
                stroke ? RoundedRectangle(cornerRadius: CornerRadius.large).stroke(
                    currentForegroundColor,
                    lineWidth: 1
                ) : nil
            )
            .padding([.top, .bottom], Padding.small)
            .font(.textSansBold, size: FontSize.buttonText)
    }
}

struct XSmallButtonStyle: ButtonStyle {
    func makeBody(configuration: Self.Configuration) -> some View {
        HStack {
            configuration.label
                .padding([.leading, .trailing], Padding.medium)
                .padding([.top, .bottom], Padding.xSmall)
                .background(
                    Capsule()
                        .stroke(
                            configuration.isPressed ? Color.XSmallButtonStyle.highlightStrokeColor : Color
                                .XSmallButtonStyle.strokeColor,
                            lineWidth: 1
                        )
                )
        }
        .foregroundColor(Color.XSmallButtonStyle.foregroundColor)
        .padding(10) // this ensures a larger, invisible tap area
        .font(.textSansBold, size: FontSize.xSmallButtonText)
    }
}

struct XSmallFilledButtonStyle: ButtonStyle {
    func makeBody(configuration: Self.Configuration) -> some View {
        HStack {
            configuration.label
                .padding([.leading, .trailing], Padding.large)
                .padding([.top, .bottom], Padding.small)
                .background(
                    Capsule()
                        .foregroundColor(Color.XSmallFilledButtonStyle.buttonColor)
                )
        }
        .foregroundColor(Color.XSmallFilledButtonStyle.textColor)
        .padding(20) // this ensures a larger, invisible tap area
        .font(.textSansBold, size: FontSize.xSmallButtonText)
    }
}

struct HelpButtonStyle: ButtonStyle {
    func makeBody(configuration: Self.Configuration) -> some View {
        let currentBackgroundColor = configuration.isPressed
            ? Color.HelpButton.backgroundPressedColor
            : Color.HelpButton.backgroundColor
        return configuration.label
            .background(currentBackgroundColor)
            .cornerRadius(CornerRadius.medium)
            .overlay(
                RoundedRectangle(cornerRadius: CornerRadius.medium)
                    .stroke(Color.HelpButton.borderColor, lineWidth: 1)
            )
    }
}

// MARK: List item modifiers

extension View {
    func recipientItemStyle() -> some View {
        return modifier(RecipientListItemStyle())
    }
}

private struct RecipientListItemStyle: ViewModifier {
    func body(content: Content) -> some View {
        return content
            .listRowSeparator(.hidden)
            .listRowBackground(
                RoundedRectangle(cornerRadius: 8)
                    .stroke(Color.RecipientListItemStyle.strokeColor, lineWidth: 1)
                    .padding([.bottom, .top], 4)
            )
    }
}

// MARK: Segmented control styling

// Native segmented controls do not support much styling, but we can override their appearance proxy to make some
// adjustments.
enum SegmentedControlAppearance {
    static func setup() {
        UISegmentedControl.appearance()
            .selectedSegmentTintColor = UIColor(Color.SegmentedControlAppearance.selectedSegmentTintColor)
        guard let font = UIFont(name: GuardianFontStyle.textSansBold.fontName,
                                size: FontSize.segmentedControlText) else { return }
        let textAttributes = [
            NSAttributedString.Key.foregroundColor: UIColor(Color.SegmentedControlAppearance.textForegroundColor),
            NSAttributedString.Key.font: font
        ]
        UISegmentedControl.appearance().setTitleTextAttributes(textAttributes, for: .normal)
        UISegmentedControl.appearance().setTitleTextAttributes(textAttributes, for: .selected)
    }
}

// MARK: Text Editor modifiers

extension TextEditor {
    func style<Style: ViewModifier>(_ style: Style) -> some View {
        ModifiedContent(content: self, modifier: style)
    }
}

struct ComposeMessageTextStyle: ViewModifier {
    func body(content: Content) -> some View {
        // This is an annoying iOS16 bug that stops
        // background colors work on text editor
        if #available(iOS 16.0, *) {
            return content
                .foregroundColor(Color.ComposeMessageTextStyle.foregroundColor)
                .scrollContentBackground(Visibility.hidden)
                .background(Color.ComposeMessageTextStyle.backgroundColor)
                .font(.textSansRegular, size: FontSize.textField)
                .background(
                    RoundedRectangle(cornerRadius: CornerRadius.small)
                        .stroke(Color.ComposeMessageTextStyle.strokeColor, lineWidth: 1)
                )
                .cornerRadius(CornerRadius.small)
        } else {
            return content
                .foregroundColor(Color.ComposeMessageTextStyle.foregroundColor)
                .background(Color.ComposeMessageTextStyle.backgroundColor)
                .font(.textSansRegular, size: FontSize.textField)
                .background(
                    RoundedRectangle(cornerRadius: CornerRadius.small)
                        .stroke(Color.ComposeMessageTextStyle.strokeColor, lineWidth: 1)
                )
                .cornerRadius(CornerRadius.small)
        }
    }
}
