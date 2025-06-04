import Foundation

/// Allows throwing an Error via a String
extension String: @retroactive Error {}
extension String: @retroactive LocalizedError {
    public var errorDescription: String? { return self }
}
