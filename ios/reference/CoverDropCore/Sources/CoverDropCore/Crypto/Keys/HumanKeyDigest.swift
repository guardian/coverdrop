import CryptoKit
import Foundation
import Sodium

/// A representation that matches the format of the key to be added to the imprint. We
/// the SHA-512 digest of the public key, truncate to the first 128 bit, and encode the result
/// in Base64 (blocks of 6 characters)
func getHumanReadableDigest(key: Sign.KeyPair.PublicKey) -> String {
    let digest = SHA512.hash(data: key)

    // Truncate to the first 128 bits
    let truncatedDigest = digest.prefix(16)

    // Encode in Base64 (and truncate to 22 characters to drop padding)
    let base64 = String(
        Data(truncatedDigest)
            .base64EncodedString(options: [])
            .prefix(22)
    )

    let chunked = base64.chunked(into: 6).joined(separator: " ")
    return chunked
}

extension String {
    func chunked(into size: Int) -> [String] {
        var chunks: [String] = []
        var startIndex = self.startIndex

        while startIndex < endIndex {
            let endIndex = index(startIndex, offsetBy: size, limitedBy: self.endIndex) ?? self.endIndex
            chunks.append(String(self[startIndex ..< endIndex]))
            startIndex = endIndex
        }

        return chunks
    }
}
