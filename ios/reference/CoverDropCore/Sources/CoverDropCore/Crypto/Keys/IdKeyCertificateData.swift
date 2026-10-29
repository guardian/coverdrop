import Foundation
import Sodium

/// The role of an identity key. The `entityName` is bound into the `IdKeyCertificateData` so that
/// e.g. a journalist identity key cannot be confused with a CoverNode identity key of the same id.
///
/// See: `entity_name` in `roles.rs`
public enum IdKeyRole: String {
    case journalistId = "journalist_id"
    case coverNodeId = "covernode_id"

    var entityName: String {
        rawValue
    }
}

/// An object that contains Key Certificate data for identity keys (journalist, covernode).
/// It contains extra fields not included in `KeyCertificateDate` tying the signature to the identity that owns the key.
public class IdKeyCertificateData: Codable {
    public var data: [UInt8]
    private static let idKeyCertDataTag = Array("ID_KEY_CERT_DATA".utf8)
    private init(data: [UInt8]) {
        self.data = data
    }

    ///
    /// Layout (length-prefixed for variable fields). See: `id_key_certificate_data.rs`
    /// ```
    /// [
    ///    "ID_KEY_CERT_DATA" (16B) |
    ///    key (32B) |
    ///    not_valid_after_be_i64 (8B) |
    ///    role_len_be_u32 (4B) |
    ///     role_name_utf8 |
    ///    id_len_be_u32 (4B) |
    ///     id_utf8
    /// ]
    /// ```
    private static func newForKeyBytes(keyBytes: [UInt8], notValidAfter: Date, role: IdKeyRole,
                                       identity: String) -> IdKeyCertificateData {
        // Use network endianess since we have to pick a cross-platform representation
        // We need to make sure it's BigEndian here in order to match the signature creation.
        let epochSeconds = Int64(notValidAfter.timeIntervalSince1970)
        let notValidAfterSecs: [UInt8] = Array(withUnsafeBytes(of: Int64(bigEndian: epochSeconds)) { Data($0) })

        let roleBytes = Array(role.entityName.utf8)
        let identityBytes = Array(identity.utf8)

        // NOTE: length prefixes prevent length-extension attacks
        let roleLenBytes: [UInt8] = Array(withUnsafeBytes(of: UInt32(roleBytes.count).bigEndian) { Data($0) })
        let identityLenBytes: [UInt8] = Array(withUnsafeBytes(of: UInt32(identityBytes.count).bigEndian) { Data($0) })

        var buf = idKeyCertDataTag
        buf.append(contentsOf: keyBytes)
        buf.append(contentsOf: notValidAfterSecs)
        buf.append(contentsOf: roleLenBytes)
        buf.append(contentsOf: roleBytes)
        buf.append(contentsOf: identityLenBytes)
        buf.append(contentsOf: identityBytes)

        return IdKeyCertificateData(data: buf)
    }

    public static func newForSigningKey(
        key: Sign.KeyPair.PublicKey,
        notValidAfter: Date,
        role: IdKeyRole,
        identity: String
    ) -> IdKeyCertificateData {
        return newForKeyBytes(keyBytes: key, notValidAfter: notValidAfter, role: role, identity: identity)
    }
}
