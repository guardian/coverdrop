import Foundation

enum PasswordGeneratorError: Error {
    case invalidChecksum
    case misspeltWords
    case passwordFormatError
}

public struct ValidPassword: Codable, Equatable {
    public let password: String

    public init(password: String) {
        self.password = password
    }

    public var words: [String] {
        password.split(separator: " ").map { String($0) }
    }

    public static func == (lhs: ValidPassword, rhs: ValidPassword) -> Bool {
        lhs.password == rhs.password
    }
}

/// A password generator to create and verify passwords and their checksums from the EFF word list
public struct PasswordGenerator {
    public var wordList: WordList = .init(words: [""])
}

public extension PasswordGenerator {
    static let shared = fromEffLargeWordlist()

    /// Create a new password generator based of the EFF large wordlist.
    private static func fromEffLargeWordlist() -> PasswordGenerator {
        if let wordlist = WordList.fromEffLargeWordlist() {
            return PasswordGenerator(wordList: wordlist)
        } else {
            return PasswordGenerator()
        }
    }

    /// The total number of words in the password generator's dictionary
    func wordsLen() -> Int {
        wordList.words.count
    }

    /// Create a new password with a given number of words with a three digit checksum.
    func generate(wordCount: Int) -> ValidPassword {
        let words: String = wordList
            .words
            .shuffled()
            .prefix(wordCount)
            .joined(separator: " ")

        return ValidPassword(password: words)
    }

    /// Generate all valid prefixes of all words. We use this for interactively checking the
    /// individual words of the passphrase as entered.
    func generatePrefixes() -> Set<String> {
        var prefixes: Set<String> = Set()
        for word in wordList.words {
            for offset in 1 ... word.count {
                prefixes.insert(String(word.prefix(offset)))
            }
        }
        return prefixes
    }

    /// Verify a password, checking it is the right format, all the words are within the dictionary,
    /// and the checksum matches.
    ///
    /// If everything is successful this function returns a `String` with the verified password embedded inside it.
    /// It is very important that you use this password in any further functions, such as key derivation, since
    /// validated
    /// passwords are transformed to lower case letters
    ///
    static func checkValid(passwordInput: String) throws -> ValidPassword {
        let wordlist = shared.wordList

        let password = passwordInput.lowercased()

        let capture = try PasswordGenerator.matchPassword(password: password)

        guard let matches: [String] = capture.get(index: 0) else {
            throw PasswordGeneratorError.passwordFormatError
        }

        guard let words: String = matches.get(index: 1) else {
            throw PasswordGeneratorError.passwordFormatError
        }

        // Find all the words which don't exist
        let invalidWords: [String] = words.split(separator: " ")
            .filter { word in
                !wordlist.words.contains(String(word))
            }.map { word in
                String(word)
            }

        if invalidWords.isEmpty {
            return ValidPassword(password: words)
        } else {
            throw PasswordGeneratorError.misspeltWords
        }
    }

    /// This matches the user inputed password, and etracts the words
    /// Given the following string "external jersey squeeze luckiness collector"
    /// We capture the "external jersey squeeze luckiness collector" as a match group
    static func matchPassword(password: String) throws -> [[String]] {
        return password.groups(for: "^([a-zA-Z ]+)$")
    }
}
