import CryptoKit
import Foundation

/// Replaced with `DeadDropSignatureData` that also covers the `created_at` field.
public struct DeadDropCertificateData: Codable {
    public var bytes: [UInt8]

    init(from: DeadDrop) {
        bytes = from.data.bytes
    }
}

public struct DeadDropSignatureData: Codable {
    public var bytes: [UInt8]

    init(from: DeadDrop) {
        // See: `journalist_to_user_dead_drop_signature_data_v2.rs`
        let epochSeconds = from.createdAt.epochSeconds
        let notValidAfterSecs: [UInt8] = Array(withUnsafeBytes(of: Int64(bigEndian: epochSeconds)) { Data($0) })

        var buf: [UInt8] = []
        buf.append(contentsOf: from.data.bytes)
        buf.append(contentsOf: notValidAfterSecs)

        bytes = CryptoKit.SHA256.hash(data: buf).bytes
    }
}
