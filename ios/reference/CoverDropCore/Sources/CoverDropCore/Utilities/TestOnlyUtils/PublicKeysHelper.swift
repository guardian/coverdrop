import Foundation
import Sodium

public enum KeysError: Error {
    case cannotFindFileError
    case cannotFindKey
}

/// This helper is used to load the public keys fixture data from disk for the purpose of unit and UI testing
/// It is located here because our tests are defined across multiple packages, and CoverDropCore is a common dependency
/// of them all
public class PublicKeysHelper {
    public let testKeys: VerifiedPublicKeys
    public let config: StaticConfig

    public static let shared = PublicKeysHelper()

    private init() {
        config = .devConfig
        // swiftlint:disable:next force_try
        let publicKeysData = try! PublicKeysHelper.readLocalKeysFile()
        // swiftlint:disable:next force_try
        let trustedOrganizationSigningKeys = try! PublicKeysHelper.readLocalTrustedOrganizationKeys(config: config)
        let verifiedPublicKeysData = VerifiedPublicKeys(
            publicKeysData: publicKeysData,
            trustedOrganizationPublicKeys: trustedOrganizationSigningKeys,
            currentTime: DateFunction.currentKeysPublishedTime()
        )
        testKeys = verifiedPublicKeysData
    }

    public static func readLocalKeysFile() throws -> PublicKeysData {
        let data = try readLocalKeysJson()
        let keys = try JSONDecoder().decode(PublicKeysData.self, from: data)
        return keys
    }

    public static func readLocalKeysJson() throws -> Data {
        let name = "001_initial_state"
        guard let resourceUrl = Bundle.module.url(
            forResource: name,
            withExtension: ".json",
            subdirectory: "vectors/create_journalists/published_keys"
        ) else { throw KeysError.cannotFindFileError }
        let data = try Data(contentsOf: resourceUrl)
        return data
    }

    public static func readLocalMultipleMessagingKeysJson() throws -> Data {
        let name = "001_initial_state"
        guard let resourceUrl = Bundle.module.url(
            forResource: name,
            withExtension: ".json",
            subdirectory: "vectors/multiple_journalists_messaging_scenario/published_keys"
        ) else { throw KeysError.cannotFindFileError }
        let data = try Data(contentsOf: resourceUrl)
        return data
    }

    public static func readLocalMessagingKeysNoDefaultJournalistJson() throws -> Data {
        let name = "001_initial_state"
        guard let resourceUrl = Bundle.module.url(
            forResource: name,
            withExtension: ".json",
            subdirectory: "vectors/messaging_scenario/published_keys"
        ) else { throw KeysError.cannotFindFileError }
        let data = try Data(contentsOf: resourceUrl)
        return data
    }

    public static func readLocalTrustedOrganizationKeys(config: StaticConfig) throws -> [TrustedOrganizationPublicKey] {
        let trustedRootKeys = try OrganizationKeysLoader.loadTrustedOrganizationPublicKeys(
            envType: config.envType,
            now: readLocalGeneratedAtFile()!
        )
        return trustedRootKeys
    }

    public static func readLocalKeypairFile(path: String) throws -> UnverifiedSignedPublicSigningKeyPairData {
        let name = path
        guard let resourceUrl = Bundle.module
            .url(forResource: name, withExtension: "keypair.json", subdirectory: "keys") else {
            throw KeysError.cannotFindFileError
        }
        let data = try Data(contentsOf: resourceUrl)
        let keyData = try JSONDecoder().decode(UnverifiedSignedPublicSigningKeyPairData.self, from: data)
        return keyData
    }

    public static func readLocalKeypairKeyOnlyFile(path: String) throws
        -> UnverifiedSignedPublicSigningKeyPairDataKeyOnly {
        let name = path
        guard let resourceUrl = Bundle.module
            .url(forResource: name, withExtension: "keypair.json", subdirectory: "keys") else {
            throw KeysError.cannotFindFileError
        }
        let data = try Data(contentsOf: resourceUrl)
        let keyData = try JSONDecoder().decode(UnverifiedSignedPublicSigningKeyPairDataKeyOnly.self, from: data)
        return keyData
    }

    public static func readLocalGeneratedAtFile() throws -> Date? {
        let name = "keys_generated_at"
        guard let resourceUrl = Bundle.module.url(forResource: name, withExtension: ".txt", subdirectory: "keys") else {
            throw KeysError.cannotFindFileError
        }
        let data = try Data(contentsOf: resourceUrl)

        if let dateString = String(data: data, encoding: .utf8) {
            return DateFormats.validateDate(date: dateString)
        }
        return nil
    }

    public static func getTestOrgKey() -> TrustedOrganizationPublicKey {
        return PublicKeysHelper.shared.testKeys.allOrganizationKeysFromAllHierarchies()
            .sorted(by: { $0.notValidAfter < $1.notValidAfter }).first!
    }

    public static func getTestCovernodeKeyHierarchy() -> [VerifiedCoverNodeKeyHierarchy] {
        let keyHierarchy = PublicKeysHelper.shared.testKeys.verifiedHierarchies.first?.verifiedCoverNodeKeyHierarchies
        return keyHierarchy!
    }

    public static func getTestCovernodeMessageKey() -> CoverNodeMessagingPublicKey? {
        let coverNodeMessagingKeys = PublicKeysHelper.shared.testKeys
            .mostRecentCoverNodeMessagingKeysFromAllHierarchies()

        guard let mostRecentCoverNodeMessagingKey = coverNodeMessagingKeys["covernode_001"] else { return nil }

        return mostRecentCoverNodeMessagingKey
    }

    public var testDefaultJournalist: JournalistData? {
        let keys = try? MessageRecipients(
            verifiedPublicKeys: PublicKeysHelper.shared.testKeys,
            excludingDefaultRecipient: false
        ).journalists
        return keys?.first(where: { value -> Bool in
            value.recipientId == "static_test_journalist"
        })
    }

    public var getTestDesk: JournalistData? {
        let keys = try? MessageRecipients(
            verifiedPublicKeys: PublicKeysHelper.shared.testKeys,
            excludingDefaultRecipient: false
        ).desks

        return keys?.first(where: { value -> Bool in
            value.recipientId == "generated_test_desk"
        })
    }

    public func getTestJournalistMessageKey() async -> JournalistMessagingPublicKey? {
        if let defaultJournalist = testDefaultJournalist {
            return PublicKeysHelper.shared.testKeys.getLatestMessagingKey(
                journalistId: defaultJournalist.recipientId
            )
        } else {
            return nil
        }
    }

    public func getTestJournalistMessageSecretKey() throws -> SecretEncryptionKey<JournalistMessaging> {
        // this is the secret key for the default recipient
        let journalistKeys = testKeys.allMessageKeysForJournalistId(journalistId: "static_test_journalist")
        guard let messageKeys = journalistKeys.first,
              let sha = messageKeys.key.key.hexStr?.prefix(8) else {
            throw KeysError.cannotFindKey
        }
        let data = try PublicKeysHelper.readLocalKeypairFile(path: "journalist_msg-\(sha)")
        return SecretEncryptionKey(key: Box.KeyPair.SecretKey(data.secretKey.bytes))
    }

    public func getTestCovernodeMessageSecretKey() throws -> SecretEncryptionKey<CoverNodeMessaging> {
        // this is the secret key for "covernode_message"
        let coverNodeKeys = testKeys.mostRecentCoverNodeMessagingKeysFromAllHierarchies()
        guard let coverNodeKey = coverNodeKeys["covernode_001"],
              let sha = coverNodeKey.key.key.hexStr?.prefix(8) else {
            throw KeysError.cannotFindKey
        }
        let data = try PublicKeysHelper.readLocalKeypairFile(path: "covernode_msg-\(sha)")
        return SecretEncryptionKey(key: Box.KeyPair.SecretKey(data.secretKey.bytes))
    }

    public func getTestUserMessageSecretKey() throws -> SecretEncryptionKey<User> {
        // this is the secret key for the default recipient
        let data = try PublicKeysHelper.readLocalKeypairKeyOnlyFile(path: "user")
        return SecretEncryptionKey(key: Box.KeyPair.SecretKey(data.secretKey.bytes))
    }

    public func getTestUserMessagePublicKey() throws -> PublicEncryptionKey<User> {
        // this is the secret key for the default recipient
        let data = try PublicKeysHelper.readLocalKeypairKeyOnlyFile(path: "user")
        return PublicEncryptionKey(key: Box.KeyPair.SecretKey(data.publicKey.key.bytes))
    }
}
