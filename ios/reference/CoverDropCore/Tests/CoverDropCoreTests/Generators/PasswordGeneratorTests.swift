@testable import CoverDropCore
import XCTest

final class PasswordGeneratorTests: XCTestCase {
    func testLoadsFileSuccessfully() throws {
        let generator = PasswordGenerator.shared
        XCTAssertEqual(
            // The large wordlist, contains enough words for passwords to be created from 5d6
            6 * 6 * 6 * 6 * 6,
            generator.wordsLen()
        )
    }

    func testMatches() throws {
        let passwordAndChecksum = "chicken cat face"
        let matches = try PasswordGenerator.matchPassword(password: passwordAndChecksum)
        XCTAssertEqual(matches[0][1], "chicken cat face")
    }

    func testRoundtrip() throws {
        let generator = PasswordGenerator.shared
        let password = generator.generate(wordCount: 5)
        let verify = try PasswordGenerator.checkValid(passwordInput: "\(password.password)")
        XCTAssertEqual(password.password, verify.password)
    }

    // This test is useful if we change the underlying checksum method, since that would pass roundtrip
    // but fail this test. If we were to accidentally change the checksum method then all our clients would
    // suddently get their passwords rejected - very bad.

    func testCheckHardcodedString() throws {
        let password = "external jersey squeeze luckiness collector"
        let verify = try PasswordGenerator.checkValid(passwordInput: password)

        XCTAssertEqual(password, "\(verify.password)")
    }

    // Since our word list is all lower case it's never valid to have an upper case character anywhere
    // in the password. As such, it feels pointless to punish our users if they accidentally capitalise
    // something. So we always lower case the password when checking it's valid.

    func testCheckHardcodedStringCaseInsensitive() throws {
        let password = "external jersey SQUEEZE luckiness collector"
        guard let validated = try? PasswordGenerator.checkValid(passwordInput: password) else {
            XCTFail("Password in test was invalid")
            return
        }

        XCTAssertEqual("external jersey squeeze luckiness collector", "\(validated.password)")
    }

    func testCheckMisssepltWord() throws {
        let password = "external jersey squeeze luckyness collector"

        XCTAssertThrowsError(try PasswordGenerator.checkValid(passwordInput: password)) { error in
            XCTAssertEqual(error as! PasswordGeneratorError, PasswordGeneratorError.misspeltWords)
        }
    }

    // This could possibly be replaced with some genuine fuzzing, but this will do for now
    func testFormatErrorsFail() throws {
        let passwords: [String] = [
            // Check that empty password is invalid
            "",
            // Check that there's no numbers mixed in with the word section
            "abc 123 abc 123",
            // Check accidental accents, useful for non-UK/US keyboard layouts?
            "wérd with accent"
        ]

        for password in passwords {
            XCTAssertThrowsError(try PasswordGenerator.checkValid(passwordInput: password)) { error in
                XCTAssertEqual(error as! PasswordGeneratorError, PasswordGeneratorError.passwordFormatError)
            }
        }
    }
}
