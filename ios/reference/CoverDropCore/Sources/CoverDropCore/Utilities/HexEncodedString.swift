import Foundation

/// A HexEncoded string is a hexidecimal string representation of a byte array
/// This is used to decode hex encoded string from our API responses
public struct HexEncodedString: Codable, Equatable {
    init(bytes: [UInt8]) {
        self.bytes = bytes
    }

    public var bytes: [UInt8]

    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        let string = try container.decode(String.self)
        guard let decodedString = string.hexStringToBytes() else {
            throw DecodingError.dataCorruptedError(
                in: container,
                debugDescription: "HexEncodedNumber does not contain a hex string"
            )
        }
        bytes = decodedString
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        guard let byteString = bytes.hexStr else {
            throw EncodingError.invalidValue(
                bytes,
                EncodingError.Context(codingPath: container.codingPath, debugDescription: "Failed to encode")
            )
        }
        try container.encode(byteString)
    }
}
