import Foundation
import Sodium

public typealias TrustedOrganizationPublicKey = SelfSignedPublicSigningKey<TrustedOrganization>
public typealias OrganizationPublicKey = SelfSignedPublicSigningKey<Organization>

public typealias CoverNodeProvisioningKey = SignedPublicSigningKey<
    CoverNodeProvisioning,
    SelfSignedPublicSigningKey<TrustedOrganization>
>
public typealias CoverNodeIdPublicKey = SignedPublicSigningKey<CoverNodeId, CoverNodeProvisioningKey>
public typealias CoverNodeMessagingPublicKey = SignedPublicEncryptionKey<CoverNodeMessaging, CoverNodeIdPublicKey>

public typealias JournalistProvisioningKey = SignedPublicSigningKey<
    JournalistProvisioning,
    SelfSignedPublicSigningKey<TrustedOrganization>
>
public typealias JournalistIdPublicKey = SignedPublicSigningKey<JournalistId, JournalistProvisioningKey>
public typealias JournalistMessagingPublicKey = SignedPublicEncryptionKey<JournalistMessaging, JournalistIdPublicKey>

public typealias UserPublicKey = PublicEncryptionKey<User>
public typealias UserSecretKey = SecretEncryptionKey<User>

public protocol SigningKey: Codable {
    var key: Sign.KeyPair.PublicKey { get set }
}

public struct SelfSignedPublicSigningKey<T: Role>: SigningKey, Codable, Equatable {
    public static func == (lhs: SelfSignedPublicSigningKey<T>, rhs: SelfSignedPublicSigningKey<T>) -> Bool {
        lhs.key == rhs.key &&
            lhs.certificate == rhs.certificate &&
            lhs.notValidAfter == rhs.notValidAfter
    }

    public var key: Sign.KeyPair.PublicKey
    public var certificate: Signature<T>
    public var notValidAfter: Date

    public init?(key: Sign.KeyPair.PublicKey, certificate: Signature<T>, notValidAfter: Date, now: Date) {
        self.key = key
        self.certificate = certificate
        self.notValidAfter = notValidAfter

        if !verify(now: now) {
            return nil
        }
    }

    public func isExpired(now: Date) -> Bool {
        return notValidAfter < now
    }

    public func verify(now: Date = DateFunction.currentTime()) -> Bool {
        let isDateValid = !isExpired(now: now)
        // The certificate for this also includes the timestamp,
        let validationCertificate = KeyCertificateData.newForSigningKey(key: key, notValidAfter: notValidAfter)

        let isKeyValid = Sodium().sign.verify(
            message: validationCertificate.data,
            publicKey: key,
            signature: certificate.certificate
        )

        return isDateValid && isKeyValid
    }
}

public class SignedPublicSigningKey<T: Role, S: SigningKey>: SigningKey, Codable {
    public var key: Sign.KeyPair.PublicKey
    public var certificate: Signature<T>
    public var notValidAfter: Date
    public init?(
        key: Sign.KeyPair.PublicKey,
        certificate: Signature<T>,
        signingKey: some SigningKey,
        notValidAfter: Date,
        now: Date
    ) {
        self.key = key
        self.certificate = certificate
        self.notValidAfter = notValidAfter

        if !verify(signingKey: signingKey, now: now) {
            return nil
        }
    }

    public func isExpired(now: Date) -> Bool {
        return notValidAfter < now
    }

    public func verify(signingKey: some SigningKey, now: Date = DateFunction.currentTime()) -> Bool {
        let isDateValid = !isExpired(now: now)
        // The certificate for this also includes the timestamp,
        let validationCertificate = KeyCertificateData.newForSigningKey(key: key, notValidAfter: notValidAfter)

        let isKeyValid = Sodium().sign.verify(
            message: validationCertificate.data,
            publicKey: signingKey.key,
            signature: certificate.certificate
        )

        return isDateValid && isKeyValid
    }

    public static func fromUnverified<R: Role, Q: SigningKey>(
        unverifiedKey: UnverifiedSignedPublicSigningKeyData,
        signingKey: Q,
        now: Date
    ) throws -> SignedPublicSigningKey<R, Q> {
        if let verifiedKey = SignedPublicSigningKey<R, Q>(
            key: Sign.KeyPair.PublicKey(unverifiedKey.key.bytes),
            certificate: Signature<R>.fromBytes(bytes: unverifiedKey.certificate.bytes),
            signingKey: signingKey,
            notValidAfter: unverifiedKey.notValidAfter.date,
            now: now
        ) {
            return verifiedKey
        } else { throw VerificationError.couldNotGetKeyFromUnverified }
    }
}

public class SignedPublicEncryptionKey<T: Role, S: SigningKey>: Codable {
    public var key: PublicEncryptionKey<T>
    public var certificate: Signature<T>
    public var notValidAfter: Date
    public init?(
        key: PublicEncryptionKey<T>,
        certificate: Signature<T>,
        signingKey: some SigningKey,
        notValidAfter: Date,
        now: Date
    ) {
        self.key = key
        self.certificate = certificate
        self.notValidAfter = notValidAfter

        if !verify(signingKey: signingKey, now: now) {
            return nil
        }
    }

    public func isExpired(now: Date) -> Bool {
        return notValidAfter < now
    }

    public func verify(signingKey: some SigningKey, now: Date = DateFunction.currentTime()) -> Bool {
        let isDateValid = !isExpired(now: now)
        // The certificate for this also includes the timestamp,
        let validationCertificate = KeyCertificateData.newForEncryptionKey(key: key, notValidAfter: notValidAfter)

        let isKeyValid = Sodium().sign.verify(
            message: validationCertificate.data,
            publicKey: signingKey.key,
            signature: certificate.certificate
        )

        return isDateValid && isKeyValid
    }

    public static func fromUnverified<R: Role, Q: SigningKey>(
        unverifiedMessageKey: UnverifiedSignedPublicEncryptionKeyData,
        signingKey: Q,
        now: Date
    ) throws -> SignedPublicEncryptionKey<R, Q> {
        if let verifiedMessageKey = SignedPublicEncryptionKey<R, Q>(
            key: PublicEncryptionKey<R>(key: unverifiedMessageKey.key.bytes),
            certificate: Signature<R>.fromBytes(bytes: unverifiedMessageKey.certificate.bytes),
            signingKey: signingKey,
            notValidAfter: unverifiedMessageKey.notValidAfter.date,
            now: now
        ) {
            return verifiedMessageKey
        } else { throw VerificationError.couldNotGetKeyFromUnverified }
    }
}
