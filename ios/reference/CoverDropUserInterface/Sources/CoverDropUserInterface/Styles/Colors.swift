import SwiftUI

/// These colours represent the Guardian palette. The colour definitons themselves should not change, but new colours
/// from the palette can of course be added as required. To change a colour in the UI, change the UI component's
/// semantic colour definition by selecting a new Guardian palette colour.
private extension Color {
    enum GuardianColor {
        static let lightGrey = Color(red: 250 / 255, green: 250 / 255, blue: 250 / 255)
        static let mediumGrey = Color(red: 226 / 255, green: 226 / 255, blue: 226 / 255)
        static let neutral0 = Color(red: 0 / 255, green: 0 / 255, blue: 0 / 255)
        static let neutral46 = Color(red: 112 / 255, green: 112 / 255, blue: 112 / 255)
        static let neutral100 = Color(red: 255 / 255, green: 255 / 255, blue: 255 / 255)
        static let brandAlt400 = Color(red: 255 / 255, green: 229 / 255, blue: 0 / 255)
        static let investigationsBase = Color(red: 57 / 255, green: 63 / 255, blue: 67 / 255)
        static let specialReport200 = Color(red: 48 / 255, green: 53 / 255, blue: 56 / 255)
        static let specialReport300 = Color(red: 63 / 255, green: 70 / 255, blue: 74 / 255)
        static let specialReport400 = Color(red: 89 / 255, green: 92 / 255, blue: 95 / 255)
        static let specialReport450 = Color(red: 157 / 255, green: 160 / 255, blue: 162 / 255)
        static let specialReport500 = Color(red: 171 / 255, green: 194 / 255, blue: 201 / 255)
        static let specialReport700 = Color(red: 228 / 255, green: 229 / 255, blue: 232 / 255)
        static let specialReport800 = Color(red: 239 / 255, green: 241 / 255, blue: 242 / 255)
        static let green500 = Color(red: 88 / 255, green: 208 / 255, blue: 139 / 255)
        static let error500 = Color(red: 255 / 255, green: 144 / 255, blue: 129 / 255)
        static let brand800 = Color(red: 193 / 255, green: 216 / 255, blue: 252 / 255)
        static let orange = Color(red: 255 / 255, green: 127 / 255, blue: 15 / 255)
    }
}

/// Coverdrop does not currently support appearances (i.e. light/dark mode). When adding support, ensure that every
/// semantic colour below returns a dynamic colour, providing a separate colour value for each appearance
extension Color {
    enum PrimaryButtonStyle {
        static let backgroundColor = GuardianColor.brandAlt400
        static let foregroundColor = GuardianColor.neutral0
    }

    enum SecondaryButtonStyle {
        static let backgroundColor = GuardianColor.investigationsBase
        static let foregroundColor = GuardianColor.neutral100
    }

    enum TertiaryButtonStyle {
        static let foregroundColor = GuardianColor.neutral100
    }

    enum LargeButtonStyle {
        static let highlightForegroundColor = GuardianColor.lightGrey
        static let highlightBackgroundColor = GuardianColor.mediumGrey
    }

    enum XSmallButtonStyle {
        static let highlightStrokeColor = GuardianColor.mediumGrey
        static let strokeColor = GuardianColor.neutral100
        static let foregroundColor = GuardianColor.neutral100
    }

    enum XSmallFilledButtonStyle {
        static let buttonColor = GuardianColor.brandAlt400
        static let textColor = GuardianColor.neutral0
    }

    enum FooterButtonStyle {
        static let foregroundColor = GuardianColor.neutral100
    }

    enum HideButtonStyle {
        static let foregroundColor = GuardianColor.neutral100
    }

    enum RecipientListItemStyle {
        static let strokeColor = GuardianColor.neutral46
    }

    enum PassphraseFieldStyle {
        static let strokeColor = GuardianColor.specialReport400
        static let backgroundColor = GuardianColor.specialReport200
        static let errorColor = GuardianColor.error500
    }

    enum CustomDivider {
        static let backgroundColor = GuardianColor.specialReport400
    }

    enum MessageMetadata {
        static let foregroundColor = GuardianColor.specialReport700
    }

    enum PassphraseTextStyle {
        static let strokeColor = GuardianColor.specialReport400
    }

    enum SelectRecipientTextStyle {
        static let strokeColor = GuardianColor.specialReport400
        static let backgroundColor = GuardianColor.specialReport200
    }

    enum RecipientTextStyle {
        static let strokeColor = GuardianColor.specialReport400
    }

    enum ListItemTitleTextStyle {
        static let foregroundColor = GuardianColor.neutral100
    }

    enum ListItemDetailTextStyle {
        static let foregroundColor = GuardianColor.neutral100
    }

    enum ProgressBarStyle {
        static let fillingColor = GuardianColor.brandAlt400
        static let fullColor = GuardianColor.error500
    }

    enum ComposeMessageTextStyle {
        static let backgroundColor = GuardianColor.specialReport200
        static let strokeColor = GuardianColor.specialReport400
        static let foregroundColor = GuardianColor.neutral100
    }

    enum SegmentedControlAppearance {
        static let selectedSegmentTintColor = GuardianColor.specialReport400
        static let textForegroundColor = GuardianColor.neutral100
    }

    enum StartCoverDropSessionView {
        static let firstLineTextForegroundColor = GuardianColor.neutral100
        static let secondLineTextForegroundColor = GuardianColor.brandAlt400
        static let foregroundColor = GuardianColor.neutral100
    }

    enum BetaBannerView {
        static let textForegroundColor = Color.black
        static let backgroundColor = GuardianColor.error500
    }

    enum PasswordBannerView {
        static let textForegroundColor = Color.black
        static let backgroundColor = GuardianColor.specialReport500
    }

    enum UserLoginView {
        static let errorMessageForegroundColor = GuardianColor.error500
        static let errorMessageStrokeColor = GuardianColor.error500
    }

    enum MessageSentView {
        static let tickIconColor = GuardianColor.green500
        static let foregroundColor = GuardianColor.neutral100
    }

    enum NewMessageView {
        static let foregroundColor = GuardianColor.neutral100
        static let messageInformationColor = GuardianColor.neutral100
        static let messageInformationStrokeColor = GuardianColor.orange
        static let messageToLongErrorColor = GuardianColor.error500
    }

    enum OnboardingView {
        static let currentPageIndicatorColor = GuardianColor.brandAlt400
        static let textForegroundColor = GuardianColor.brandAlt400
    }

    enum UserNewSessionView {
        static let wordListBackgroundColor = GuardianColor.investigationsBase
        static let errorColor = GuardianColor.error500
    }

    enum HeaderView {
        static let fillColor = GuardianColor.investigationsBase
        static let backgroundColor = GuardianColor.specialReport200
        static let arrowColor = GuardianColor.neutral100
    }

    enum JournalistNewMessageView {
        static let navigationBarBackgroundColor = GuardianColor.investigationsBase
        static let scrollviewBackgroundColor = GuardianColor.investigationsBase
        static let textEditorForegroundColor = GuardianColor.neutral100
        static let textEditorBackgroundColor = GuardianColor.specialReport200
        static let textEditorStrokeColor = GuardianColor.neutral100
        static let messageListBackgroundColor = GuardianColor.investigationsBase
        static let messageListForegroundColor = GuardianColor.neutral100
        static let messageViewCurrentUserColor = GuardianColor.specialReport200
        static let messageViewUnselectedUserColor = GuardianColor.specialReport200
        static let tickIconColor = GuardianColor.green500
    }

    enum InboxView {
        static let activeMessageBorderColor = GuardianColor.specialReport400
        static let activeMessageSubHeaderColor = GuardianColor.neutral100
        static let activeMessageRecipientColor = GuardianColor.brandAlt400
        static let lastMessageBubbleColor = GuardianColor.specialReport800
        static let lastMessageColor = GuardianColor.specialReport800
        static let previousMessagesTitleColor = GuardianColor.neutral100
        static let previousMessagesSubheaderColor = GuardianColor.neutral100
        static let previousMessageRecipientColor = GuardianColor.brandAlt400
        static let deleteMessageButtonColor = GuardianColor.neutral100
        static let aboutButtonColor = GuardianColor.neutral100
    }

    enum HelpExample {
        static let highlightColor = GuardianColor.brandAlt400
        static let backgroundColor = GuardianColor.specialReport200
        static let borderColor = GuardianColor.neutral46
    }

    enum HelpBlockQuote {
        static let highlightColor = GuardianColor.brandAlt400
    }

    enum HelpList {
        static let bulletColor = GuardianColor.brandAlt400
    }

    enum HelpButton {
        static let backgroundColor = GuardianColor.investigationsBase
        static let backgroundPressedColor = GuardianColor.neutral46
        static let borderColor = GuardianColor.neutral46
    }

    enum HelpDivider {
        static let lineColor = GuardianColor.neutral46
    }

    enum ChevronButtonList {
        static let dividerColor = GuardianColor.neutral46
        static let backgroundColor = GuardianColor.specialReport200
        static let chevronColor = GuardianColor.neutral100
    }
}
