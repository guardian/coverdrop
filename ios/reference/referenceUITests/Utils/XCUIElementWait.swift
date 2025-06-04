import Foundation
import XCTest

// This was lifted from :
// https://gist.github.com/ryanpato/f9b2a44b76045c8a97e64f716103a735#file-xcuielement-wait01-swift
extension XCUIElement {
    @discardableResult
    func wait(
        until expression: @escaping (XCUIElement) -> Bool,
        timeout: TimeInterval = 20,
        message: @autoclosure () -> String = "",
        file: StaticString = #file,
        line: UInt = #line
    ) -> Self {
        if expression(self) {
            return self
        }

        let predicate = NSPredicate { _, _ in
            expression(self)
        }

        let expectation = XCTNSPredicateExpectation(predicate: predicate, object: nil)

        let result = XCTWaiter().wait(for: [expectation], timeout: timeout)

        if result != .completed {
            XCTFail(
                message().isEmpty ? "expectation not matched after waiting" : message(),
                file: file,
                line: line
            )
        }

        return self
    }
}
