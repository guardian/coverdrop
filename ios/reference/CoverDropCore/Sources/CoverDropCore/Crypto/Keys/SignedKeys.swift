import Foundation
import Sodium

public typealias TrustedOrganizationPublicKey = SelfSignedPublicSigningKey<TrustedOrganization>
public typealias OrganizationPublicKey = SelfSignedPublicSigningKey<Organization>

public typealias CoverNodeProvisioningKey = SignedPublicSigningKey<
    CoverNodeProvisioning,
    SelfSignedPublicSigningKey<TrustedOrganization>
>
public typealias CoverNodeIdPublicKey = SignedIdentityPublicSigningKey<CoverNodeId, CoverNodeProvisioningKey>
public typealias CoverNodeMessagingPublicKey = SignedPublicEncryptionKey<CoverNodeMessaging, CoverNodeIdPublicKey>

public typealias JournalistProvisioningKey = SignedPublicSigningKey<
    JournalistProvisioning,
    SelfSignedPublicSigningKey<TrustedOrganization>
>
public typealias JournalistIdPublicKey = SignedIdentityPublicSigningKey<JournalistId, JournalistProvisioningKey>
public typealias JournalistMessagingPublicKey = SignedPublicEncryptionKey<JournalistMessaging, JournalistIdPublicKey>

public typealias UserPublicKey = PublicEncryptionKey<User>
public typealias UserSecretKey = SecretEncryptionKey<User>

public protocol SigningKey: Codable {
    var key: Sign.KeyPair.PublicKey { get set }
}

/// Shared core of all `verify()` implementations below: checks expiry, verifies the legacy
/// `legacyCertificate` against `legacyCertificateData`, and, if present, verifies `signature` against
/// `signatureCertificateData`. The two data payloads are only ever different for identity keys, where the
/// signature additionally binds the role and identity.
private func verifyCertificateAndSignature(
    now: Date,
    notValidAfter: Date,
    signingKey: Sign.KeyPair.PublicKey,
    legacyCertificateData: [UInt8],
    legacyCertificate: [UInt8],
    signatureData: [UInt8],
    signature: [UInt8]?
) -> Bool {
    let isDateValid = notValidAfter >= now

    let isCertificateValid = Sodium().sign.verify(
        message: legacyCertificateData,
        publicKey: signingKey,
        signature: legacyCertificate
    )

    let isSignatureValid = signature.map {
        Sodium().sign.verify(
            message: signatureData,
            publicKey: signingKey,
            signature: $0
        )
    } ?? true

    return isDateValid && isCertificateValid && isSignatureValid
}

public struct SelfSignedPublicSigningKey<T: Role>: SigningKey, Codable, Equatable {
    public static func == (lhs: SelfSignedPublicSigningKey<T>, rhs: SelfSignedPublicSigningKey<T>) -> Bool {
        lhs.key == rhs.key &&
            lhs.certificate == rhs.certificate &&
            lhs.signature == rhs.signature &&
            lhs.notValidAfter == rhs.notValidAfter
    }

    public var key: Sign.KeyPair.PublicKey
    public var certificate: Signature<T>
    public var signature: Signature<T>?
    public var notValidAfter: Date

    public init?(
        key: Sign.KeyPair.PublicKey,
        certificate: Signature<T>,
        signature: Signature<T>?,
        notValidAfter: Date,
        now: Date
    ) {
        self.key = key
        self.certificate = certificate
        self.signature = signature
        self.notValidAfter = notValidAfter

        guard verify(now: now) else { return nil }
    }

    public func verify(now: Date = DateFunction.currentTime()) -> Bool {
        let validationCertificate = KeyCertificateData.newForSigningKey(key: key, notValidAfter: notValidAfter)

        return verifyCertificateAndSignature(
            now: now,
            notValidAfter: notValidAfter,
            signingKey: key,
            legacyCertificateData: validationCertificate.data,
            legacyCertificate: certificate.certificate,
            signatureData: validationCertificate.data,
            signature: signature?.certificate
        )
    }
}

public class SignedPublicSigningKey<T: Role, S: SigningKey>: SigningKey, Codable {
    public var key: Sign.KeyPair.PublicKey
    // Legacy certificate. Remove in https://github.com/guardian/coverdrop-internal/issues/4200
    public var certificate: Signature<T>
    public var signature: Signature<T>?
    public var notValidAfter: Date

    public init?(
        key: Sign.KeyPair.PublicKey,
        certificate: Signature<T>,
        signature: Signature<T>?,
        signingKey: some SigningKey,
        notValidAfter: Date,
        now: Date
    ) {
        self.key = key
        self.certificate = certificate
        self.signature = signature
        self.notValidAfter = notValidAfter

        guard verify(signingKey: signingKey, now: now) else { return nil }
    }

    public func verify(signingKey: some SigningKey, now: Date = DateFunction.currentTime()) -> Bool {
        let validationCertificate = KeyCertificateData.newForSigningKey(key: key, notValidAfter: notValidAfter)

        return verifyCertificateAndSignature(
            now: now,
            notValidAfter: notValidAfter,
            signingKey: signingKey.key,
            legacyCertificateData: validationCertificate.data,
            legacyCertificate: certificate.certificate,
            signatureData: validationCertificate.data,
            signature: signature?.certificate
        )
    }

    public static func fromUnverified<R: Role, Q: SigningKey>(
        unverifiedKey: UnverifiedSignedPublicSigningKeyData,
        signingKey: Q,
        now: Date
    ) throws -> SignedPublicSigningKey<R, Q> {
        if let verifiedKey = SignedPublicSigningKey<R, Q>(
            key: Sign.KeyPair.PublicKey(unverifiedKey.key.bytes),
            certificate: Signature<R>.fromBytes(bytes: unverifiedKey.certificate.bytes),
            signature: unverifiedKey.signature.map { Signature<R>.fromBytes(bytes: $0.bytes) },
            signingKey: signingKey,
            notValidAfter: unverifiedKey.notValidAfter.date,
            now: now
        ) {
            return verifiedKey
        } else {
            throw VerificationError.couldNotGetKeyFromUnverified
        }
    }
}

/// Dedicated type for identity keys (journalist and covernode) so that the identity-bound `verify`/`fromUnverified`
/// is the only construction path available.
public class SignedIdentityPublicSigningKey<T: IdentityRole, S: SigningKey>: SigningKey, Codable {
    public var key: Sign.KeyPair.PublicKey
    // Legacy certificate. Remove in https://github.com/guardian/coverdrop-internal/issues/4200
    public var certificate: Signature<T>
    public var signature: Signature<T>?
    public var notValidAfter: Date

    public init?(
        key: Sign.KeyPair.PublicKey,
        certificate: Signature<T>,
        signature: Signature<T>?,
        signingKey: some SigningKey,
        identity: String,
        notValidAfter: Date,
        now: Date
    ) {
        self.key = key
        self.certificate = certificate
        self.signature = signature
        self.notValidAfter = notValidAfter

        guard verify(signingKey: signingKey, identity: identity, now: now) else { return nil }
    }

    public func verify(signingKey: some SigningKey, identity: String, now: Date = DateFunction.currentTime()) -> Bool {
        let legacyCertificate = KeyCertificateData.newForSigningKey(key: key, notValidAfter: notValidAfter)
        let idKeyCertificate = IdKeyCertificateData.newForSigningKey(
            key: key,
            notValidAfter: notValidAfter,
            role: T.idKeyRole,
            identity: identity
        )

        return verifyCertificateAndSignature(
            now: now,
            notValidAfter: notValidAfter,
            signingKey: signingKey.key,
            legacyCertificateData: legacyCertificate.data,
            legacyCertificate: certificate.certificate,
            signatureData: idKeyCertificate.data,
            signature: signature?.certificate
        )
    }

    public static func fromUnverified<Q: SigningKey>(
        unverifiedKey: UnverifiedSignedPublicSigningKeyData,
        signingKey: Q,
        identity: String,
        now: Date
    ) throws -> SignedIdentityPublicSigningKey<T, Q> {
        if let verifiedKey = SignedIdentityPublicSigningKey<T, Q>(
            key: Sign.KeyPair.PublicKey(unverifiedKey.key.bytes),
            certificate: Signature<T>.fromBytes(bytes: unverifiedKey.certificate.bytes),
            signature: unverifiedKey.signature.map { Signature<T>.fromBytes(bytes: $0.bytes) },
            signingKey: signingKey,
            identity: identity,
            notValidAfter: unverifiedKey.notValidAfter.date,
            now: now
        ) {
            return verifiedKey
        } else {
            throw VerificationError.couldNotGetKeyFromUnverified
        }
    }
}

public class SignedPublicEncryptionKey<T: Role, S: SigningKey>: Codable {
    public var key: PublicEncryptionKey<T>
    public var certificate: Signature<T>
    public var signature: Signature<T>?
    public var notValidAfter: Date
    public init?(
        key: PublicEncryptionKey<T>,
        certificate: Signature<T>,
        signature: Signature<T>?,
        signingKey: some SigningKey,
        notValidAfter: Date,
        now: Date
    ) {
        self.key = key
        self.certificate = certificate
        self.signature = signature
        self.notValidAfter = notValidAfter

        guard verify(signingKey: signingKey, now: now) else { return nil }
    }

    public func isExpired(now: Date) -> Bool {
        return notValidAfter < now
    }

    public func verify(signingKey: some SigningKey, now: Date = DateFunction.currentTime()) -> Bool {
        let validationCertificate = KeyCertificateData.newForEncryptionKey(key: key, notValidAfter: notValidAfter)

        return verifyCertificateAndSignature(
            now: now,
            notValidAfter: notValidAfter,
            signingKey: signingKey.key,
            legacyCertificateData: validationCertificate.data,
            legacyCertificate: certificate.certificate,
            signatureData: validationCertificate.data,
            signature: signature?.certificate
        )
    }

    public static func fromUnverified<R: Role, Q: SigningKey>(
        unverifiedMessageKey: UnverifiedSignedPublicEncryptionKeyData,
        signingKey: Q,
        now: Date
    ) throws -> SignedPublicEncryptionKey<R, Q> {
        if let verifiedMessageKey = SignedPublicEncryptionKey<R, Q>(
            key: PublicEncryptionKey<R>(key: unverifiedMessageKey.key.bytes),
            certificate: Signature<R>.fromBytes(bytes: unverifiedMessageKey.certificate.bytes),
            signature: unverifiedMessageKey.signature.map { Signature<R>.fromBytes(bytes: $0.bytes) },
            signingKey: signingKey,
            notValidAfter: unverifiedMessageKey.notValidAfter.date,
            now: now
        ) {
            return verifiedMessageKey
        } else {
            throw VerificationError.couldNotGetKeyFromUnverified
        }
    }
}
