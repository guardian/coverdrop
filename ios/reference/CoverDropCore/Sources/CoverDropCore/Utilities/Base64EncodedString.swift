import Foundation

/// A Base64EncodedString string is a base64 string representation of a byte array
/// This is used to decode base64 encoded strings from our API responses
public struct Base64EncodedString: Codable, Equatable {
    // This init is here to facilitate testing
    init(bytes: [UInt8]) {
        self.bytes = bytes
    }

    public var bytes: [UInt8]

    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        let string = try container.decode(String.self)
        guard let decodedString = string.base64Decode() else {
            throw DecodingError.dataCorruptedError(in: container, debugDescription: "base64 decoding failed")
        }
        bytes = decodedString
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        guard let byteString = bytes.base64Encode() else {
            throw EncodingError.invalidValue(
                bytes,
                EncodingError.Context(codingPath: container.codingPath, debugDescription: "Failed to encode to base64")
            )
        }
        try container.encode(byteString)
    }
}
