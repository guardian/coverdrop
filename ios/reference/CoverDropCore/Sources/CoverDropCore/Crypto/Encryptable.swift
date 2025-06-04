import Foundation
import Sodium
/// A structure that can be **directly** encrypted.
///
/// This is useful when you don't want to incur some kind of serializion costs, such as
/// increased message size or performance penalties.

public protocol Encryptable {
    associatedtype ObjectType = Self

    func asUnencryptedBytes() -> [UInt8]
    static func fromUnencryptedBytes(bytes: [UInt8]) throws -> ObjectType
}

extension String: Encryptable {
    enum EncryptableStringError: Error {
        case failedFromUnencryptedBytes
    }

    public func asBytes() -> [UInt8] {
        utf8.map { UInt8($0) }
    }

    public func asUnencryptedBytes() -> [UInt8] {
        asBytes()
    }

    public static func fromUnencryptedBytes(bytes: [UInt8]) throws -> String {
        if let string = String(bytes: bytes, encoding: .utf8) {
            return string
        } else { throw EncryptableStringError.failedFromUnencryptedBytes }
    }

    public func hexStringToBytes() -> [UInt8]? {
        return Sodium().utils.hex2bin(self)
    }
}

extension [UInt8]: Encryptable {
    public func asUnencryptedBytes() -> [UInt8] {
        self
    }

    public static func fromUnencryptedBytes(bytes: [UInt8]) -> [UInt8] {
        bytes
    }
}

extension UnlockedSecretData: Encryptable {
    public func asUnencryptedBytes() -> [UInt8] {
        do {
            let jsonEncoder = JSONEncoder()
            jsonEncoder.outputFormatting = .sortedKeys
            let jsonEncodedState = try jsonEncoder.encode(self)
            return Array(Data(jsonEncodedState))
        } catch {
            return []
        }
    }

    public static func fromUnencryptedBytes(bytes: [UInt8]) throws -> UnlockedSecretData {
        return try JSONDecoder().decode(UnlockedSecretData.self, from: Data(bytes))
    }
}
