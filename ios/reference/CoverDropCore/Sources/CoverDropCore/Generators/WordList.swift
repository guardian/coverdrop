import Foundation

enum WordListError: Error {
    case invalidWordListLine
}

public struct WordList {
    public let words: [String]
}

public extension WordList {
    static func parseEffLargeWordlist(text: String) throws -> [String] {
        let words: [String] = try text.split(separator: "\n")
            // Filter empty lines
            .filter { line in
                !line.isEmpty
            }
            // Each line of the passphrase file has a number, a tab seperator, then the word
            // We split on the tab, and return the word part
            .map { line in
                if let word = line.split(separator: "\t").last {
                    return String(word)
                } else {
                    throw WordListError.invalidWordListLine
                }
            }

        return words
    }

    /// Create a new password generator based of the EFF large wordlist.
    static func fromEffLargeWordlist() -> WordList? {
        let wordlistPath = "eff_large_wordlist"
        guard let resourceUrl = Bundle.module.url(forResource: wordlistPath, withExtension: ".txt") else { return nil }
        do {
            let wordlist = try String(contentsOf: resourceUrl, encoding: .utf8)
            let words = try parseEffLargeWordlist(text: wordlist)
            return WordList(words: words)
        } catch {
            return nil
        }
    }
}
