import Foundation
import Sodium

typealias JournalistToUserMessage = TwoPartyBox<[UInt8]>

enum VerificationDeadDropError: Error {
    case deadDropDataWrongSize
}

/// This is a list of verified dead drops
/// Each dead drop has been verified before passed into this container
public struct VerifiedDeadDrops {
    var deadDrops: [VerifiedDeadDrop]
}

public extension VerifiedDeadDrops {
    /// Converts unverfied DeadDropData loaded from the user dead drop API
    /// into VerifiedDeadDropData by checking that each dead drop
    /// has been signed by the CoverNodeId Public Key
    /// - Parameters:
    ///   - deadDrops: A list of DeadDrops
    ///   - signingPk: The signing public key - in this case a CoverNodeIdPublicKey used to verify the dead drops
    /// - Returns: a VerifiedDeadDrops struct - only dead drops that are verified are returned
    static func fromAllDeadDropData(deadDrops: DeadDropData, verifiedKeys: VerifiedPublicKeys) -> VerifiedDeadDrops {
        let verifiedDeadDrops = allVerifiedDeadDropsFromDeadDropData(deadDrops: deadDrops, verifiedKeys: verifiedKeys)

        return VerifiedDeadDrops(deadDrops: verifiedDeadDrops)
    }

    static func allVerifiedDeadDropsFromDeadDropData(
        deadDrops: DeadDropData,
        verifiedKeys: VerifiedPublicKeys
    ) -> [VerifiedDeadDrop] {
        var verifiedDeadDrops: [VerifiedDeadDrop] = []

        let coverNodeIdKeys: [String: [CoverNodeIdPublicKey]] = verifiedKeys.getAllCoverNodeIdKeysInAllHierarchies()

        for deadDrop in deadDrops.deadDrops {
            for coverNodeIdKey in coverNodeIdKeys.values {
                for key in coverNodeIdKey {
                    if let verfiedDeadDrop = VerifiedDeadDrop(unverifiedDeadDrop: deadDrop, signingPk: key) {
                        verifiedDeadDrops.append(verfiedDeadDrop)
                    }
                }
            }
        }

        return verifiedDeadDrops
    }
}

/// An individual dead drop that has been verified against the CoverNodeId Public Signing Key.
public struct VerifiedDeadDrop {
    var id: Int
    var data: [JournalistToUserMessage]
    var publishedDate: Date

    /// Verifies the signature of the `PublishedJournalistToUserDeadDrop` using the `signingKey`.
    ///
    /// During the migration phase, we only check the `signature` field if it has a meaningful value.
    /// Otherwise, we fallback to the "legacy" check against the `cert` field. This fallback
    /// behaviour is only temporary and should be removed once the migration is complete, see #2998.
    init?(unverifiedDeadDrop: DeadDrop, signingPk: CoverNodeIdPublicKey) {
        let unverifiedDeadDropCertificateData = DeadDropCertificateData(from: unverifiedDeadDrop)
        let unverifiedDeadDropSignatureData = DeadDropSignatureData(from: unverifiedDeadDrop)
        let hasMeaningfulSignature = VerifiedDeadDrop.isMeaningfulSignature(signature: unverifiedDeadDrop.signature)

        do {
            let verified = if hasMeaningfulSignature {
                VerifiedDeadDrop.verify(
                    signingPk: signingPk,
                    data: unverifiedDeadDropSignatureData.bytes,
                    signature: Signature.fromBytes(bytes: unverifiedDeadDrop.signature?.bytes ?? [])
                )
            } else {
                // if the signature is not meaningful, we fallback to the cert check; see #2998
                VerifiedDeadDrop.verify(
                    signingPk: signingPk,
                    data: unverifiedDeadDropCertificateData.bytes,
                    signature: Signature.fromBytes(bytes: unverifiedDeadDrop.cert?.bytes ?? [])
                )
            }

            if verified {
                let parsedDeadDropData = try VerifiedDeadDrop.parseDeadDropData(data: unverifiedDeadDrop.data.bytes)
                let verifiedCreatedAt = unverifiedDeadDrop.createdAt.date

                // Check the deaddrop publish date is not more that 1 week in the future which might be caused
                // by dramatic clock skew between us and the API. In that case, we ignore and hope for better
                // alignment for the next try.
                if DateFunction.currentTime().distance(to: verifiedCreatedAt) > 7 * 24 * 3600 {
                    return nil
                }

                // All checks passed
                id = unverifiedDeadDrop.id
                data = parsedDeadDropData
                publishedDate = verifiedCreatedAt
            } else { return nil }
        } catch { return nil }
    }

    /// Returns `true` if the signature is present and has at least one non-zero byte. In that case
    /// we can assume that  we have a dead-drop with the new signature scheme and we should
    /// check this signature instead of the legacy `cert` field. See #2998.
    static func isMeaningfulSignature(signature: HexEncodedString?) -> Bool {
        guard let signature = signature else {
            // missing signatures are never meaningful
            return false
        }

        for byte in signature.bytes where byte != 0x00 {
            // if the signature contains at least one non-zero byte, we can
            // assume that it is a meaningful signature
            return true
        }

        return false
    }

    /// Parse the verified dead drop data into a list of JournalistToUserMessages
    /// The entire dead drop data byte array is a fixed sized list of TwoPartyBoxes
    /// https://github.com/guardian/coverdrop/blob/main/docs/protocol_messages.md#journalist-to-user-message
    /// This function splits the dead drop data into a list of JournalistToUserMessages
    /// - Parameter data: raw dead drop data
    /// - Returns:  a list of JournalistToUserMessages
    /// - throws: If any of chunked dead drop data is not the correct size
    static func parseDeadDropData(data: [UInt8]) throws -> [JournalistToUserMessage] {
        let deadDropFromData = data.chunked(into: Constants.journalistToUserEncryptedMessageLen)

        return try deadDropFromData.map { unverifiedDeadDrop in
            if unverifiedDeadDrop.count != Constants.journalistToUserEncryptedMessageLen {
                throw VerificationDeadDropError.deadDropDataWrongSize
            }
            let journalistToUserMessage: JournalistToUserMessage = TwoPartyBox<PaddedCompressedString>
                .fromVecUnchecked(bytes: unverifiedDeadDrop)
            return journalistToUserMessage
        }
    }

    /// Verifies a dead drop data byte array against its certificate and CoverNodeId Public Key
    /// - Parameters:
    ///   - signingPk: the CoverNodeId Public Key for the Private Key used to sign the dead drop data
    ///   - data: the dead drop data
    ///   - signature: the signature provided alongside the dead drop data in the API respose
    /// - Returns: `true` if the verfication is sucessful, `false` if verification fails
    static func verify(
        signingPk: CoverNodeIdPublicKey,
        data: [UInt8],
        signature: Signature<CoverNodeId>
    ) -> Bool {
        return Sodium().sign.verify(message: data, publicKey: signingPk.key, signature: signature.certificate)
    }
}
