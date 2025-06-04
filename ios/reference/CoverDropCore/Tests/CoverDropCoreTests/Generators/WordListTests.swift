@testable import CoverDropCore
import XCTest

final class WordListTests: XCTestCase {
    func testParseWordList() throws {
        let text = """
        66656\tzombie
        66661\tzone
        66662\tzoning
        66663\tzookeeper
        66664\tzoologist
        66665\tzoology
        66666\tzoom
        """

        let data = try WordList.parseEffLargeWordlist(text: text)

        XCTAssertTrue(data.contains("zoom"))
    }
}
