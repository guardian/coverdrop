import SwiftUI

struct HelpHeadline: View {
    var text: String
    var level: HeadlineLevel

    var body: some View {
        switch level {
        case .headline1:
            Text(text)
                .textStyle(LargeTitleStyle())
                .padding(.bottom, Padding.large)
        case .headline2:
            Text(text)
                .textStyle(GuardianHeaderTextStyle())
        case .headline3:
            Text(text)
                .fontWeight(.bold)
        }
    }
}

struct HelpList: View {
    var lines: [String]

    var body: some View {
        VStack(alignment: .leading) {
            ForEach(lines, id: \.self) { line in
                HStack(alignment: .top) {
                    Text("•")
                        .fontWeight(.bold)
                        .foregroundColor(Color.HelpList.bulletColor)
                        .padding(.leading, Padding.xSmall)
                    Text(line)
                }
            }
        }
    }
}

struct HelpBlockQuote: View {
    var text: String
    var authorName: String
    var tagLine: String

    var body: some View {
        HStack {
            Rectangle()
                .fill(Color.HelpBlockQuote.highlightColor)
                .frame(width: 3)
            VStack(alignment: .leading) {
                Text(text)
                Text(authorName).fontWeight(.medium).padding(.top, Padding.small)
                Text(tagLine)
            }
        }
    }
}

struct HelpExample: View {
    var attributedString: AttributedString

    init(textWithHighlights: TextWithHighlights) {
        var attributedString = AttributedString()
        let highlightFont: Font = .body.weight(.semibold)

        for run in textWithHighlights.runs {
            switch run {
            case let .none(text):
                attributedString.append(AttributedString(text))
            case let .highlighted(text):
                var highlightedPart = AttributedString(text)
                highlightedPart.foregroundColor = Color.HelpExample.highlightColor
                highlightedPart.font = highlightFont
                attributedString.append(highlightedPart)
            }
        }
        self.attributedString = attributedString
    }

    var body: some View {
        HStack(alignment: .top) {
            Text(attributedString).padding(.all, Padding.medium)
            Spacer()
        }
        .background(Color.HelpExample.backgroundColor)
        .cornerRadius(CornerRadius.medium)
        .overlay(
            RoundedRectangle(cornerRadius: CornerRadius.medium)
                .stroke(Color.HelpExample.borderColor, lineWidth: 1)
        )
    }
}

struct HelpPassphraseBoxes: View {
    var words: [String]

    var body: some View {
        VStack(spacing: 0) {
            Text("Example of a passphrase")
                .font(.body)
                .fontWeight(.medium)
                .padding(.bottom, Padding.small)
            VStack {
                VStack(spacing: Padding.medium) {
                    ForEach(words, id: \.self) { word in
                        Text(word)
                            .textStyle(PassphraseTextStyle())
                            .padding(.horizontal, Padding.medium)
                    }
                }.padding(.vertical, Padding.medium)
            }.background(Color.HelpExample.backgroundColor)
                .cornerRadius(CornerRadius.medium)
                .overlay(
                    RoundedRectangle(cornerRadius: CornerRadius.medium)
                        .stroke(Color.HelpExample.borderColor, lineWidth: 1)
                )
        }
    }
}

struct HelpButton: View {
    var action: () -> Void
    var firstLine: String
    var secondLine: String

    var body: some View {
        Button(action: action) {
            HStack(alignment: .top, spacing: Padding.small) {
                Image(.iconRoundInfo)
                    .frame(width: 24, height: 18)
                    .padding(.top, 1)

                VStack(alignment: .leading) {
                    Text(firstLine)
                        .textStyle(BodyStyle(bottomPadding: 0))
                        .fontWeight(.semibold)
                    Text(secondLine).textStyle(BodyStyle(bottomPadding: Padding.xSmall))
                }
                Spacer()
            }.padding(.all, 8)
        }.buttonStyle(HelpButtonStyle())
    }
}

struct HelpDivider: View {
    var body: some View {
        Rectangle()
            .fill(Color.HelpDivider.lineColor)
            .frame(height: 1)
            .padding(.vertical, Padding.large)
    }
}
