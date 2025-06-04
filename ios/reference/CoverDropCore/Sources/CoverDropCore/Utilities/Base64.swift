import Foundation

public extension Data {
    /// Same as ``Data(base64Encoded:)``, but adds padding automatically
    /// (if missing, instead of returning `nil`).
    static func fromBase64(_ encoded: String) -> Data? {
        // Prefixes padding-character(s) (if needed).
        var encoded = encoded
        let remainder = encoded.count % 4
        if remainder > 0 {
            encoded = encoded.padding(
                toLength: encoded.count + 4 - remainder,
                withPad: "=", startingAt: 0
            )
        }

        // Finally, decode.
        return Data(base64Encoded: encoded)
    }
}

/**
 * Decodes a Base64 [String] as a [ByteArray].
 */
extension String {
    func base64Decode() -> [UInt8]? {
        guard let data = Data.fromBase64(self) else {
            return nil
        }

        return Array(data)
    }
}

/**
 * Encodes a [ByteArray] as a Base64 [String].
 */
extension [UInt8] {
    func base64Encode() -> String? {
        let data = Data(self)
        return data.base64EncodedString()
    }
}
