public struct UnverifiedSignedPublicSigningKeyPairData: Codable, Equatable {
    public static func == (lhs: UnverifiedSignedPublicSigningKeyPairData,
                           rhs: UnverifiedSignedPublicSigningKeyPairData) -> Bool {
        return lhs.secretKey == rhs.secretKey
    }

    public init(publicKey: UnverifiedSignedPublicSigningKeyData, secretKey: HexEncodedString) {
        self.publicKey = publicKey
        self.secretKey = secretKey
    }

    var publicKey: UnverifiedSignedPublicSigningKeyData
    var secretKey: HexEncodedString

    enum CodingKeys: String, CodingKey {
        case publicKey = "public_key"
        case secretKey = "secret_key"
    }
}

// swiftlint:disable type_name
public struct UnverifiedSignedPublicSigningKeyPairDataKeyOnly: Codable, Equatable {
    public static func == (
        lhs: UnverifiedSignedPublicSigningKeyPairDataKeyOnly,
        rhs: UnverifiedSignedPublicSigningKeyPairDataKeyOnly
    ) -> Bool {
        return lhs.secretKey == rhs.secretKey &&
            lhs.publicKey == rhs.publicKey
    }

    public init(publicKey: KeyPairDataKeyOnly, secretKey: HexEncodedString) {
        self.publicKey = publicKey
        self.secretKey = secretKey
    }

    var publicKey: KeyPairDataKeyOnly
    var secretKey: HexEncodedString

    enum CodingKeys: String, CodingKey {
        case publicKey = "public_key"
        case secretKey = "secret_key"
    }
}

// swiftlint:enable type_name

public struct KeyPairDataKeyOnly: Codable, Equatable {
    var key: HexEncodedString
    public init(key: HexEncodedString) {
        self.key = key
    }
}
