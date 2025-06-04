import SwiftUI

enum HeadlineLevel {
    case headline1, headline2, headline3
}

enum HelpScreenComponent {
    case headline(text: String, level: HeadlineLevel)
    case text(String)
    case listItem(String)
    case divider
    case space
    case example(TextWithHighlights)
    case blockQuote(text: String, authorName: String, tagLine: String)
    case button(firstLine: String, secondLine: String, identifier: String)
    case passphraseBoxes(words: [String])
}

enum HelpScreenMarkupParsingError: Error {
    case invalidButton
    case invalidBlockQuote
}

enum AttributedRun: Equatable {
    case none(text: String)
    case highlighted(text: String)
}

struct TextWithHighlights {
    var runs: [AttributedRun]
}

class HelpScreenMarkupParser {
    private let highlightedTextParser = HighlightedTextParser()

    func parseHelpScreenMarkup(markup: String) throws -> [HelpScreenComponent] {
        let markup = markup.trimmingCharacters(in: .newlines)
        let sections = markup.components(separatedBy: "\n\n")
        return try sections.map { section in
            if section.hasPrefix("# ") {
                return .headline(text: section.replacingOccurrences(of: "# ", with: ""), level: .headline1)
            } else if section.hasPrefix("## ") {
                return .headline(text: section.replacingOccurrences(of: "## ", with: ""), level: .headline2)
            } else if section.hasPrefix("### ") {
                return .headline(text: section.replacingOccurrences(of: "### ", with: ""), level: .headline3)
            } else if section.hasPrefix("- ") {
                return .listItem(section.replacingOccurrences(of: "- ", with: ""))
            } else if section == "DIVIDER" {
                return .divider
            } else if section == "SPACE" {
                return .space
            } else if section.hasPrefix("EXAMPLE") {
                return try parseExampleBox(section)
            } else if section.hasPrefix("BLOCKQUOTE") {
                return try parseBlockQuote(section)
            } else if section.hasPrefix("BUTTON") {
                return try parseButton(section)
            } else if section.hasPrefix("PASSPHRASE_BOXES") {
                let words = section.replacingOccurrences(of: "PASSPHRASE_BOXES ", with: "").components(separatedBy: " ")
                return .passphraseBoxes(words: words)
            } else {
                return .text(section)
            }
        }
    }

    func parseButton(_ section: String) throws -> HelpScreenComponent {
        let buttonSection = section.replacingOccurrences(of: "BUTTON\n", with: "")
        let buttonLines = buttonSection.components(separatedBy: "\n")
        if buttonLines.count != 3 {
            throw HelpScreenMarkupParsingError.invalidButton
        }
        return .button(
            firstLine: buttonLines[0],
            secondLine: buttonLines[1],
            identifier: buttonLines[2].trimmingCharacters(in: .whitespaces)
        )
    }

    func parseBlockQuote(_ section: String) throws -> HelpScreenComponent {
        let blockQuoteText = section.replacingOccurrences(of: "BLOCKQUOTE\n", with: "")
        let quoteParts = blockQuoteText.components(separatedBy: "\n")
        if quoteParts.count != 3 {
            throw HelpScreenMarkupParsingError.invalidBlockQuote
        }
        return .blockQuote(
            text: quoteParts[0],
            authorName: quoteParts[1],
            tagLine: quoteParts[2]
        )
    }

    func parseExampleBox(_ section: String) throws -> HelpScreenComponent {
        let exampleText = section.replacingOccurrences(of: "EXAMPLE\n", with: "")
        let annotatedText = highlightedTextParser.parseIntoTextWithHighlights(exampleText)
        return .example(annotatedText)
    }
}

/**
  * Parses a string of the form "text ~highlighted~ more text ~more highlights~.", where
  * the text between ~ characters will be attributed with the given highlight color.
 */
class HighlightedTextParser {
    func parseIntoTextWithHighlights(_ text: String) -> TextWithHighlights {
        let parts = text.split(separator: "~")
        var attributedRuns = [AttributedRun]()

        for (index, part) in parts.enumerated() {
            if index % 2 == 0 {
                attributedRuns.append(.none(text: String(part)))
            } else {
                attributedRuns.append(.highlighted(text: String(part)))
            }
        }

        return TextWithHighlights(runs: attributedRuns)
    }
}
