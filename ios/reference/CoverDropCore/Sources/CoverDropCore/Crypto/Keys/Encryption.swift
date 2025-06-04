import Foundation
import Sodium

enum EncryptionError: Error {
    case invalidPublicKeyHex
    case failedToDecrypt
    case failedToEncrypt
    case failedToGenerateKeys
}

public protocol EncryptionKey {
    var key: [UInt8] { get }
}

public class EncryptionKeypair<T: Role>: Codable {
    public var publicKey: PublicEncryptionKey<T>

    public var secretKey: SecretEncryptionKey<T>

    /// Create a keypair to be used for encryption
    public static func generateEncryptionKeypair<R: Role>() throws -> EncryptionKeypair<R> {
        if let keypair = Sodium().box.keyPair() {
            let secretKey = keypair.secretKey
            let publicKey = keypair.publicKey

            return EncryptionKeypair<R>(
                publicKey: PublicEncryptionKey<R>(key: publicKey),
                secretKey: SecretEncryptionKey<R>(key: secretKey)
            )
        } else {
            throw EncryptionError.failedToGenerateKeys
        }
    }

    public init(publicKey: PublicEncryptionKey<T>, secretKey: SecretEncryptionKey<T>) {
        self.secretKey = secretKey
        self.publicKey = publicKey
    }
}

public struct PublicEncryptionKey<T: Role>: Codable, EncryptionKey, Equatable {
    public var key: Box.KeyPair.PublicKey

    public init(key: Box.KeyPair.PublicKey) {
        self.key = key
    }
}

public extension PublicEncryptionKey {
    func new(key: Box.KeyPair.PublicKey) -> Self {
        return PublicEncryptionKey<T>(key: key)
    }

    func toBytes() -> [UInt8] {
        return key.asUnencryptedBytes()
    }

    static func from_bytes(bytes: [UInt8]) throws -> Self {
        if bytes.count != Sodium().box.PublicKeyBytes {
            throw EncryptionError.invalidPublicKeyHex
        } else {
            return PublicEncryptionKey<T>(key: Box.KeyPair.PublicKey(bytes))
        }
    }
}

public struct SecretEncryptionKey<T: Role>: Codable, EncryptionKey {
    public var key: Box.KeyPair.SecretKey
}

public extension SecretEncryptionKey {
    func new(key: Box.KeyPair.SecretKey) -> Self {
        return SecretEncryptionKey(key: key)
    }
}

public struct Signature<T: Role>: Codable, Equatable {
    public var certificate: [UInt8]
    private init(certificate: [UInt8]) {
        self.certificate = certificate
    }

    public static func fromBytes(bytes: [UInt8]) -> Self {
        Signature(certificate: bytes)
    }
}
