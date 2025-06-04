import Foundation
import Sodium

/// An object that contains Key Certificate data
public class KeyCertificateData: Codable {
    public var data: [UInt8]
    private init(data: [UInt8]) {
        self.data = data
    }

    /// This takes a key and date and coverts it into a `KeyCertificateData` object
    /// We covert the date into a Unix timestamp and represent this a Big Endian unsigned 64bit interger.
    /// We then make a 40 byte array of the key bytes followed by the date bytes.
    /// ```
    ///   +------------+--------+
    ///   |    key     | date   |
    ///   +------------+--------+
    ///      ^              ^
    ///      |              |
    ///     32B             8B
    /// ```
    /// This same method is used to sign the key, so you can only validate the key with its signature by including
    /// expiry date.
    /// - Parameters:
    ///   - keyBytes: the byte representation of a PublicEncryptionKey<T>
    ///   - notValidAfter: the expiry date of the key
    /// - Returns: `KeyCertificateData`
    private static func newForKeyBytes(keyBytes: [UInt8], notValidAfter: Date) -> KeyCertificateData {
        // Use network endianess since we have to pick a cross-platform representation
        // We need to make sure its BigEndian here !
        let epochSeconds = Int64(notValidAfter.timeIntervalSince1970)
        let notValidAfterSecs: [UInt8] = Array(withUnsafeBytes(of: Int64(bigEndian: epochSeconds)) { Data($0) })

        var buf = keyBytes
        buf.append(contentsOf: notValidAfterSecs)

        return KeyCertificateData(data: buf)
    }

    public static func newForEncryptionKey<T: Role>(key: PublicEncryptionKey<T>,
                                                    notValidAfter: Date) -> KeyCertificateData {
        return newForKeyBytes(keyBytes: key.key, notValidAfter: notValidAfter)
    }

    public static func newForSigningKey(key: Sign.KeyPair.PublicKey, notValidAfter: Date) -> KeyCertificateData {
        return newForKeyBytes(keyBytes: key, notValidAfter: notValidAfter)
    }
}
