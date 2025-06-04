import Foundation
import Sodium

enum VerificationError: Error {
    case couldNotGetKeyFromUnverified
    case noJournalistKeysPresent
    case couldNotVerifyOrganizationKey
}

/// This contains a verified set of public keys, journalist profiles, and the default journalistId
/// The public keys are verified, the profiles and default journalist are implicitly trusted
public struct VerifiedPublicKeys {
    public var journalistProfiles: [JournalistProfile]
    public let defaultJournalistId: String?
    public let verifiedHierarchies: [VerifiedPublicKeysHierarchy]

    /// Creates a `VerifiedPublicKeys` from `PublicKeysData`
    /// - Parameters:
    ///   - publicKeysData: An instance of PublicKeysData, this can be a response from the `public-keys` API endpoint,
    /// or a cached version from disk.
    ///   - trustedOrganizationPublicKeys: A list of `TrustedOrganizationPublicKey`, these are got from the app package
    /// bundle, so baked into the build.
    ///   - currentTime: the current time the app is running in
    /// - Returns: An instance of `VerifiedPublicKeys`
    init(
        publicKeysData: PublicKeysData,
        trustedOrganizationPublicKeys: [TrustedOrganizationPublicKey],
        currentTime: Date
    ) {
        let verifiedHierarchies: [VerifiedPublicKeysHierarchy] = publicKeysData.keys.compactMap { keyHierarchy in
            VerifiedPublicKeysHierarchy(
                keyHierarchy: keyHierarchy,
                trustedOrganizationPublicKeys: trustedOrganizationPublicKeys,
                currentTime: currentTime
            )
        }
        let defaultJournalistId = publicKeysData.defaultJournalistId

        journalistProfiles = publicKeysData.journalistProfiles
        self.defaultJournalistId = defaultJournalistId
        self.verifiedHierarchies = verifiedHierarchies
    }

    public func allOrganizationKeysFromAllHierarchies() -> [TrustedOrganizationPublicKey] {
        var trustedOrganizationPublicKeys: [TrustedOrganizationPublicKey] = []
        for verifiedPublicKeysHierarchies in verifiedHierarchies {
            trustedOrganizationPublicKeys.append(verifiedPublicKeysHierarchies.organizationPublicKey)
        }
        return trustedOrganizationPublicKeys
    }

    private func allCoverNodeKeysFromAllHierarchies() -> [VerifiedCoverNodeKeyHierarchy] {
        var verifiedCoverNodeKeyHierarchies: [VerifiedCoverNodeKeyHierarchy] = []
        for verifiedPublicKeysHierarchies in verifiedHierarchies {
            for coverNodeKeys in verifiedPublicKeysHierarchies.verifiedCoverNodeKeyHierarchies {
                verifiedCoverNodeKeyHierarchies.append(coverNodeKeys)
            }
        }
        return verifiedCoverNodeKeyHierarchies
    }

    public func mostRecentCoverNodeMessagingKeysFromAllHierarchies()
        -> [CoverNodeIdentity: CoverNodeMessagingPublicKey] {
        var coverNodeInstanceMessageKeys: [CoverNodeIdentity: [CoverNodeMessagingPublicKey]] = [:]
        let allCoverNodes = allCoverNodeKeysFromAllHierarchies()

        // get all the messaging from all hierarchies keys first
        for coverNodes in allCoverNodes {
            for (coverNodeId, verifiedCoverNodeKeysFamilies) in coverNodes.idPublicKeys {
                for verifiedCoverNodeKeysFamily in verifiedCoverNodeKeysFamilies {
                    if var currentMessageKeys = coverNodeInstanceMessageKeys[coverNodeId] {
                        currentMessageKeys.append(contentsOf: verifiedCoverNodeKeysFamily.msg)
                        coverNodeInstanceMessageKeys.updateValue(currentMessageKeys, forKey: coverNodeId)
                    } else {
                        coverNodeInstanceMessageKeys.updateValue(verifiedCoverNodeKeysFamily.msg, forKey: coverNodeId)
                    }
                }
            }
        }

        // now get the most recent key regardless of hierarchy
        var coverNodeInstanceMessageKey: [CoverNodeIdentity: CoverNodeMessagingPublicKey] = [:]

        for (coverNodeId, messagePublicKeys) in coverNodeInstanceMessageKeys {
            if let recentKey = messagePublicKeys.max(by: { $0.notValidAfter < $1.notValidAfter }) {
                coverNodeInstanceMessageKey.updateValue(recentKey, forKey: coverNodeId)
            }
        }
        return coverNodeInstanceMessageKey
    }

    /// This gets all the public keys for each journalist regardless of the key hierarchy the keys are in.
    /// This is required because when we try and decode incoming messages, it is possible for them to be encrypted with
    /// older keys, so we must try all keys.
    /// - Returns: An Dictionary of `String` -> `[VerifiedJournalistPublicKeysGroup]` where string is the journalist id.
    private func allPublicKeysForJournalistsFromAllHierarchies() -> [String: [VerifiedJournalistPublicKeysGroup]] {
        var journalistPublicKeys: [String: [VerifiedJournalistPublicKeysGroup]] = [:]

        for verifiedPublicKeysHierarchies in verifiedHierarchies {
            for verifiedPublicKeysHierarchy in verifiedPublicKeysHierarchies.verifiedJournalistKeyHierarchies {
                for (journalistId, publicKey) in verifiedPublicKeysHierarchy.keys {
                    if var existing = journalistPublicKeys[journalistId] {
                        existing.append(contentsOf: publicKey)
                        journalistPublicKeys.updateValue(existing, forKey: journalistId)
                    } else {
                        journalistPublicKeys.updateValue(publicKey, forKey: journalistId)
                    }
                }
            }
        }
        return journalistPublicKeys
    }

    private func allPublicKeysForJournalistId(journalistId: String) -> [VerifiedJournalistPublicKeysGroup]? {
        if let journalistPublicKeys = allPublicKeysForJournalistsFromAllHierarchies()[journalistId],
           journalistPublicKeys.isEmpty == false {
            return journalistPublicKeys

        } else {
            return nil
        }
    }

    public func allMessageKeysForJournalistId(journalistId: String) -> [JournalistMessagingPublicKey] {
        guard let journalistPublicKeyData = allPublicKeysForJournalistId(journalistId: journalistId) else {
            return []
        }
        let allMessagingKeys: [JournalistMessagingPublicKey] = journalistPublicKeyData.flatMap { keyData in
            keyData.msg
        }
        return allMessagingKeys
    }

    /// This gets all the coverNode Id keys for each CoverNode instance regardless of the key hierarchy the keys are in.
    /// - Returns: An Dictionary of `CoverNodeInstanceId` -> `[CoverNodeIdPublicKey]` where string is the coverNode
    /// instance id.
    func getAllCoverNodeIdKeysInAllHierarchies() -> [CoverNodeIdentity: [CoverNodeIdPublicKey]] {
        var coverNodeIdKeys: [String: [CoverNodeIdPublicKey]] = [:]

        for verifiedKeys in verifiedHierarchies {
            for coverNodeKeysFamily in verifiedKeys.verifiedCoverNodeKeyHierarchies {
                if let newKeys: [String: [CoverNodeIdPublicKey]] = coverNodeKeysFamily.getAllCoverNodeIdKeys() {
                    coverNodeIdKeys.merge(newKeys, uniquingKeysWith: { current, new in
                        current + new
                    })
                }
            }
        }
        return coverNodeIdKeys
    }

    public func getJournalistKeyDataForJournalistId(journalistId: String) -> JournalistData? {
        guard let profileData = journalistProfiles.first(where: { $0.id == journalistId }) else {
            return nil
        }

        return JournalistData(
            recipientId: journalistId,
            displayName: profileData.displayName,
            isDesk: profileData.isDesk,
            recipientDescription: profileData.description,
            tag: RecipientTag(tag: profileData.tag.bytes),
            visibility: profileData.status == .visible ? .visible : .hidden
        )
    }

    public func getLatestMessagingKey(journalistId: String) -> JournalistMessagingPublicKey? {
        let messageKeys = allMessageKeysForJournalistId(journalistId: journalistId)
        return messageKeys.max { $0.notValidAfter < $1.notValidAfter }
    }
}

/// This is a verified Public Keys Hierarchy. This means that all the keys have be verified against their signing keys
/// all the way to the root `TrustedOrganizationPublicKey`
public struct VerifiedPublicKeysHierarchy {
    public var verifiedJournalistKeyHierarchies: [VerifiedJournalistPublicKeyHierarchy]
    public var organizationPublicKey: TrustedOrganizationPublicKey
    public var verifiedCoverNodeKeyHierarchies: [VerifiedCoverNodeKeyHierarchy]

    /// This gets all the public keys for each journalist for this key hierarchy.
    /// - Returns: An Dictionary of `String` -> `[VerifiedJournalistPublicKeysGroup]` where string is the journalist id.
    public func allPublicKeysForJournalists() -> [String: [VerifiedJournalistPublicKeysGroup]] {
        var keys: [String: [VerifiedJournalistPublicKeysGroup]] = [:]

        for journalistPublicKeys in verifiedJournalistKeyHierarchies {
            for (journalistId, publicKey) in journalistPublicKeys.keys {
                keys.updateValue(publicKey, forKey: journalistId)
            }
        }
        return keys
    }

    /// Failable initialiser for a VerifiedPublicKeysHierarchy.
    /// - Parameters:
    ///   - keyHierarchy: The Key Hierarchy to verify
    ///   - trustedOrganizationPublicKeys: A list of `TrustedOrganizationPublicKey` to try and verify with
    ///   - currentTime: currentTime: the current time the app is running in
    /// - Returns: Returns a VerifiedPublicKeysHierarchy if successfully verified, or fails if it could not verify.
    init?(keyHierarchy: KeyHierarchy, trustedOrganizationPublicKeys: [TrustedOrganizationPublicKey],
          currentTime: Date) {
        let unverifiedOrganizationPublicKey = SelfSignedPublicSigningKey<Organization>(
            key: Sign.KeyPair.PublicKey(keyHierarchy.organizationPublicKey.key.bytes),
            certificate: Signature<Organization>.fromBytes(
                bytes: keyHierarchy.organizationPublicKey.certificate.bytes
            ),
            notValidAfter: keyHierarchy.organizationPublicKey.notValidAfter.date,
            now: currentTime
        )

        guard let orgPublicKey = unverifiedOrganizationPublicKey,
              let trustedOrganizationPublicKey: TrustedOrganizationPublicKey = VerifiedPublicKeysHierarchy
              .verifyOrganizationPublicKey(
                  orgPk: orgPublicKey,
                  trustedOrgPks: trustedOrganizationPublicKeys
              ) else {
            return nil
        }

        do {
            let coverNodesHierarchies: [VerifiedCoverNodeKeyHierarchy] = try VerifiedPublicKeysHierarchy
                .verifyCoverNodeHierarchy(
                    keyHierarchy: keyHierarchy,
                    trustedOrganizationPublicKey: trustedOrganizationPublicKey,
                    currentTime: currentTime
                )

            let verifiedJournalistPublicKeysHierarchry: [VerifiedJournalistPublicKeyHierarchy] =
                try VerifiedPublicKeysHierarchy.verifyJournalistKeysHierarchy(
                    keyHierarchy: keyHierarchy,
                    trustedOrganizationPublicKey: trustedOrganizationPublicKey,
                    currentTime: currentTime
                )

            if keyHierarchy.journalists.isEmpty {
                return nil
            } else {
                verifiedJournalistKeyHierarchies = verifiedJournalistPublicKeysHierarchry
                organizationPublicKey = trustedOrganizationPublicKey
                verifiedCoverNodeKeyHierarchies = coverNodesHierarchies
            }
        } catch {
            return nil
        }
    }

    /// Verify a list of coverNode Key Hierarchies from a KeyHierarchy
    /// - Parameters:
    ///   - keyHierarchy: The Key Hierarchy to verify
    ///   - trustedOrganizationPublicKeys: A list of `TrustedOrganizationPublicKey` to try and verify with
    ///   - currentTime: currentTime: the current time the app is running in
    /// - Returns: A list of VerifiedCoverNodeKeyHierarchy  if succesfully verified
    /// - Throws: An error if verification was unsuccessful
    public static func verifyCoverNodeHierarchy(
        keyHierarchy: KeyHierarchy,
        trustedOrganizationPublicKey: TrustedOrganizationPublicKey,
        currentTime: Date
    ) throws -> [VerifiedCoverNodeKeyHierarchy] {
        return try keyHierarchy.coverNodes.map { coverNodeHierarchy in

            let coverNodeProvisioningKey = coverNodeHierarchy.provisioningPublicKey

            let verifiedCoverNodeProvisioningKey: CoverNodeProvisioningKey = try CoverNodeProvisioningKey
                .fromUnverified(
                    unverifiedKey: coverNodeProvisioningKey,
                    signingKey: trustedOrganizationPublicKey,
                    now: currentTime
                )

            var coverNodeInstances: [CoverNodeIdentity: [VerifiedCoverNodeKeysFamily]] = [:]

            for (coverNodeId, coverNodeKeysFamilies) in coverNodeHierarchy.covernodes {
                let coverNodeKeys = try coverNodeKeysFamilies.map { family in
                    let verifiedCoverNodeIdKey: CoverNodeIdPublicKey = try CoverNodeIdPublicKey.fromUnverified(
                        unverifiedKey: family.idPk,
                        signingKey: verifiedCoverNodeProvisioningKey,
                        now: currentTime
                    )
                    let coverNodeMessageKeys: [CoverNodeMessagingPublicKey] = try extractCoverNodeMessageKeys(
                        keysGroup: family.msgPks,
                        verifiedCoverNodeIdKey: verifiedCoverNodeIdKey,
                        now: currentTime
                    )
                    return VerifiedCoverNodeKeysFamily(id: verifiedCoverNodeIdKey, msg: coverNodeMessageKeys)
                }
                coverNodeInstances.updateValue(coverNodeKeys, forKey: coverNodeId)
            }
            let verifiedCoverNodeKeyHierarchy: VerifiedCoverNodeKeyHierarchy = .init(
                idPublicKeys: coverNodeInstances,
                provisioningKey: verifiedCoverNodeProvisioningKey
            )
            return verifiedCoverNodeKeyHierarchy
        }
    }

    /// Verify a list of Journalist Public Key Hierarchies from a KeyHierarchy
    /// - Parameters:
    ///   - keyHierarchy: The Key Hierarchy to verify
    ///   - trustedOrganizationPublicKeys: A list of `TrustedOrganizationPublicKey` to try and verify with
    ///   - currentTime: currentTime: the current time the app is running in
    /// - Returns: A list of VerifiedJournalistPublicKeyHierarchy  if succesfully verified
    /// - Throws: An error if verification was unsuccessful
    private static func verifyJournalistKeysHierarchy(
        keyHierarchy: KeyHierarchy,
        trustedOrganizationPublicKey: TrustedOrganizationPublicKey,
        currentTime: Date
    ) throws -> [VerifiedJournalistPublicKeyHierarchy] {
        try keyHierarchy.journalists.map { journalistHierarchy in

            let journalistProvisioningKey = journalistHierarchy.provisioningPublicKey

            let verifiedJournalistProvisioningKey: JournalistProvisioningKey = try JournalistProvisioningKey
                .fromUnverified(
                    unverifiedKey: journalistProvisioningKey,
                    signingKey: trustedOrganizationPublicKey,
                    now: currentTime
                )

            var journalists: [String: [VerifiedJournalistPublicKeysGroup]] = [:]

            for (journalistId, journalistKeysFamily) in journalistHierarchy.journalists {
                let verifiedJournalistPublicKeysGroup: [VerifiedJournalistPublicKeysGroup] =
                    try verifyJournalistPublicKeys(
                        keysGroup: journalistKeysFamily,
                        journalistProvisioningKey: verifiedJournalistProvisioningKey,
                        now: currentTime
                    )
                journalists.updateValue(verifiedJournalistPublicKeysGroup, forKey: journalistId)
            }
            return VerifiedJournalistPublicKeyHierarchy(
                keys: journalists,
                provisioningKey: verifiedJournalistProvisioningKey
            )
        }
    }

    /// Verifies an untrusted `OrganizationPublicKey` against a list of `TrustedOrganizationPublicKey`
    /// - Parameters:
    ///   - orgPk: A `OrganizationPublicKey` from the `public-keys` API response
    ///   - trustedOrgPks: A list of `TrustedOrganizationPublicKey` loaded from the app bundle
    /// - Returns: A `TrustedOrganizationPublicKey` if verification was succesful
    public static func verifyOrganizationPublicKey(
        orgPk: OrganizationPublicKey,
        trustedOrgPks: [TrustedOrganizationPublicKey]
    ) -> TrustedOrganizationPublicKey? {
        if let trustedKey = trustedOrgPks.first(where: { trustedKey in
            orgPk.key == trustedKey.key
        }) {
            return trustedKey
        } else {
            return nil
        }
    }

    /// This verifies a list of `JournalistKeysFamily` against the `JournalistProvisioningKey`
    /// - Parameters:
    ///   - keysGroup: A list of `JournalistProvisioningKey`
    ///   - journalistProvisioningKey: A `JournalistProvisioningKey`
    ///   - now: the current time in the app
    /// - Returns: a list of `VerifiedJournalistPublicKeysGroup`
    /// - Throws: if verification was unsuccesful
    private static func verifyJournalistPublicKeys(
        keysGroup: [JournalistKeysFamily],
        journalistProvisioningKey: JournalistProvisioningKey,
        now: Date
    ) throws -> [VerifiedJournalistPublicKeysGroup] {
        return try keysGroup.map { key in
            let unverifiedIdKey: UnverifiedSignedPublicSigningKeyData = key.idPk
            let verifiedIdKey: JournalistIdPublicKey = try JournalistIdPublicKey.fromUnverified(
                unverifiedKey: unverifiedIdKey,
                signingKey: journalistProvisioningKey,
                now: now
            )

            let verifiedMessageKeys: [JournalistMessagingPublicKey] = try verifyJournalistMessageKeys(
                keysGroup: key.msgPks,
                verifiedJournalistIdPublicKey: verifiedIdKey,
                now: now
            )

            return VerifiedJournalistPublicKeysGroup(id: verifiedIdKey,
                                                     msg: verifiedMessageKeys)
        }
    }

    /// Verifies a list of `UnverifiedSignedPublicEncryptionKeyData` against the supplied `JournalistIdPublicKey`
    /// - Parameters:
    ///   - keysGroup: a list of `UnverifiedSignedPublicEncryptionKeyData`
    ///   - verifiedJournalistIdPublicKey: a verified `JournalistIdPublicKey`
    ///   - now: the current time in the app
    /// - Returns: a list of `JournalistMessagingPublicKey`
    private static func verifyJournalistMessageKeys(
        keysGroup: [UnverifiedSignedPublicEncryptionKeyData],
        verifiedJournalistIdPublicKey: JournalistIdPublicKey,
        now: Date
    ) throws -> [JournalistMessagingPublicKey] {
        return try keysGroup.map { messageKey in
            let verifiedMessageKey: JournalistMessagingPublicKey = try JournalistMessagingPublicKey.fromUnverified(
                unverifiedMessageKey: messageKey,
                signingKey: verifiedJournalistIdPublicKey,
                now: now
            )
            return verifiedMessageKey
        }
    }

    /// Verifies a list of `UnverifiedSignedPublicEncryptionKeyData` against the supplied `CoverNodeIdPublicKey`
    /// - Parameters:
    ///   - keysGroup: a list of `UnverifiedSignedPublicEncryptionKeyData`
    ///   - verifiedCoverNodeIdKey: a verified `CoverNodeIdPublicKey`
    ///   - now: the current time in the app
    /// - Returns: a list of `CoverNodeMessagingPublicKey`
    private static func extractCoverNodeMessageKeys(
        keysGroup: [UnverifiedSignedPublicEncryptionKeyData],
        verifiedCoverNodeIdKey: CoverNodeIdPublicKey,
        now: Date
    ) throws -> [CoverNodeMessagingPublicKey] {
        return try keysGroup.map { messageKey in
            let verifiedMessageKey: CoverNodeMessagingPublicKey = try CoverNodeMessagingPublicKey.fromUnverified(
                unverifiedMessageKey: messageKey,
                signingKey: verifiedCoverNodeIdKey,
                now: now
            )
            return verifiedMessageKey
        }
    }
}

/// A `VerifiedJournalistPublicKeysGroup`
public struct VerifiedJournalistPublicKeysGroup {
    let id: JournalistIdPublicKey
    let msg: [JournalistMessagingPublicKey]
}

/// A `VerifiedJournalistPublicKeyHierarchy`
public struct VerifiedJournalistPublicKeyHierarchy {
    public let keys: [String: [VerifiedJournalistPublicKeysGroup]]
    public let provisioningKey: JournalistProvisioningKey
}

public typealias CoverNodeIdentity = String

/// A `VerifiedCoverNodeKeysFamily`
public struct VerifiedCoverNodeKeysFamily {
    public let id: CoverNodeIdPublicKey
    public let msg: [CoverNodeMessagingPublicKey]
}

/// A `VerifiedCoverNodeKeyHierarchy`
public struct VerifiedCoverNodeKeyHierarchy {
    public let idPublicKeys: [String: [VerifiedCoverNodeKeysFamily]]
    public let provisioningKey: CoverNodeProvisioningKey

    /// This gets all the coverNode Id keys for each CoverNode instance in the current hierarchy
    /// - Returns: An Dictionary of `String` -> `[CoverNodeIdPublicKey]` where string is the coverNode instance id.
    public func getAllCoverNodeIdKeys() -> [CoverNodeIdentity: [CoverNodeIdPublicKey]]? {
        var coverNodeKeys: [CoverNodeIdentity: [CoverNodeIdPublicKey]] = [:]
        for (coverNodeInstanceId, coverNodeKeysFamilies) in idPublicKeys {
            for coverNodeKeysFamily in coverNodeKeysFamilies {
                let mostRecentKey = coverNodeKeysFamily.id
                coverNodeKeys[coverNodeInstanceId, default: []].append(mostRecentKey)
            }
        }
        return coverNodeKeys
    }
}
