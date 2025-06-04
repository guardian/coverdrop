import Foundation
import Sodium

enum PublicKeysError: Error {
    case failedToGetData
}

public struct PublicKeysData: Codable, Equatable {
    public static func == (lhs: PublicKeysData, rhs: PublicKeysData) -> Bool {
        return lhs.keys == rhs.keys &&
            lhs.defaultJournalistId == rhs.defaultJournalistId &&
            lhs.journalistProfiles == rhs.journalistProfiles
    }

    public var journalistProfiles: [JournalistProfile]
    public var keys: [KeyHierarchy]
    public let defaultJournalistId: String?

    enum CodingKeys: String, CodingKey {
        case journalistProfiles = "journalist_profiles"
        case keys
        case defaultJournalistId = "default_journalist_id"
    }
}

public struct KeyHierarchy: Codable, Equatable {
    public var organizationPublicKey: UnverifiedSignedPublicSigningKeyData
    public var journalists: [JournalistKeyHierarchy]
    public var coverNodes: [CoverNodesKeyHierarchy]

    enum CodingKeys: String, CodingKey {
        case organizationPublicKey = "org_pk"
        case journalists
        case coverNodes = "covernodes"
    }

    public static func == (lhs: KeyHierarchy, rhs: KeyHierarchy) -> Bool {
        return lhs.organizationPublicKey == rhs.organizationPublicKey &&
            lhs.journalists == rhs.journalists &&
            lhs.coverNodes == rhs.coverNodes
    }
}

public struct CoverNodesKeyHierarchy: Codable, Equatable {
    public var provisioningPublicKey: UnverifiedSignedPublicSigningKeyData
    public var covernodes: [String: [CoverNodeKeysFamily]]

    enum CodingKeys: String, CodingKey {
        case provisioningPublicKey = "provisioning_pk"
        case covernodes
    }

    public static func == (lhs: CoverNodesKeyHierarchy, rhs: CoverNodesKeyHierarchy) -> Bool {
        return lhs.provisioningPublicKey == rhs.provisioningPublicKey &&
            lhs.covernodes == rhs.covernodes
    }
}

public struct JournalistKeyHierarchy: Codable, Equatable {
    public var provisioningPublicKey: UnverifiedSignedPublicSigningKeyData
    public var journalists: [String: [JournalistKeysFamily]]

    enum CodingKeys: String, CodingKey {
        case provisioningPublicKey = "provisioning_pk"
        case journalists
    }

    public static func == (lhs: JournalistKeyHierarchy, rhs: JournalistKeyHierarchy) -> Bool {
        return lhs.provisioningPublicKey == rhs.provisioningPublicKey &&
            lhs.journalists == rhs.journalists
    }
}

public struct PublicKeysFamily: Codable, Equatable {
    public static func == (lhs: PublicKeysFamily, rhs: PublicKeysFamily) -> Bool {
        return lhs.idPk == rhs.idPk &&
            lhs.msgPks == rhs.msgPks
    }

    public var idPk: UnverifiedSignedPublicSigningKeyData
    public var msgPks: [UnverifiedSignedPublicEncryptionKeyData]

    enum CodingKeys: String, CodingKey {
        case idPk = "id_pk"
        case msgPks = "msg_pks"
    }
}

public typealias CoverNodeKeysFamily = PublicKeysFamily
public typealias JournalistKeysFamily = PublicKeysFamily

public enum JournalistStatus: String, Codable {
    case visible = "VISIBLE"
    case hiddenFromUi = "HIDDEN_FROM_UI"
    case hiddenFromResponse = "HIDDEN_FROM_RESPONSE"
}

public struct JournalistProfile: Codable, Equatable {
    public init(
        id: String,
        displayName: String,
        sortName: String,
        description: String,
        isDesk: Bool,
        tag: HexEncodedString,
        status: JournalistStatus
    ) {
        self.id = id
        self.displayName = displayName
        self.sortName = sortName
        self.description = description
        self.isDesk = isDesk
        self.tag = tag
        self.status = status
    }

    public var id: String
    public var displayName: String
    public var sortName: String
    public var description: String
    public var isDesk: Bool
    public var tag: HexEncodedString
    public var status: JournalistStatus

    enum CodingKeys: String, CodingKey {
        case id
        case displayName = "display_name"
        case sortName = "sort_name"
        case description
        case isDesk = "is_desk"
        case tag
        case status
    }
}

public struct UnverifiedSignedPublicEncryptionKeyData: Codable, Equatable {
    public init(key: HexEncodedString, certificate: HexEncodedString, notValidAfter: RFC3339DateTimeString) {
        self.key = key
        self.certificate = certificate
        self.notValidAfter = notValidAfter
    }

    public var key: HexEncodedString
    public var certificate: HexEncodedString
    public var notValidAfter: RFC3339DateTimeString

    enum CodingKeys: String, CodingKey {
        case key
        case certificate
        case notValidAfter = "not_valid_after"
    }
}

public struct UnverifiedSignedPublicSigningKeyData: Codable, Equatable {
    public static func == (lhs: UnverifiedSignedPublicSigningKeyData,
                           rhs: UnverifiedSignedPublicSigningKeyData) -> Bool {
        return lhs.key == rhs.key &&
            lhs.certificate == rhs.certificate &&
            lhs.notValidAfter == rhs.notValidAfter
    }

    public init(key: HexEncodedString, certificate: HexEncodedString, notValidAfter: RFC3339DateTimeString) {
        self.key = key
        self.certificate = certificate
        self.notValidAfter = notValidAfter
    }

    var key: HexEncodedString
    var certificate: HexEncodedString
    public var notValidAfter: RFC3339DateTimeString

    enum CodingKeys: String, CodingKey {
        case key
        case certificate
        case notValidAfter = "not_valid_after"
    }
}
