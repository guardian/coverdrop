import Foundation
import Sodium

enum OrganizationKeysLoader {
    public static func loadTrustedOrganizationPublicKeys(
        envType: EnvType,
        now: Date
    ) throws -> [TrustedOrganizationPublicKey] {
        let subpath: EnvType = envType
        let resourcePaths: [String] = Bundle.module.paths(
            forResourcesOfType: "json",
            inDirectory: "organization_keys/\(subpath)/"
        )

        let keys: [TrustedOrganizationPublicKey] = try resourcePaths.compactMap { fullPath in
            // As `Bundle.module.paths` returns the full path, we just want to get the filename
            let fileName = URL(fileURLWithPath: fullPath).lastPathComponent
            let fileNameWithoutExtension = (fileName as NSString).deletingPathExtension
            let resourceUrlOption = Bundle.module.url(
                forResource: fileNameWithoutExtension,
                withExtension: ".json",
                subdirectory: "organization_keys/\(subpath)/"
            )
            if let resourceUrl = resourceUrlOption {
                let data = try Data(contentsOf: resourceUrl)
                let keyData = try JSONDecoder().decode(UnverifiedSignedPublicSigningKeyData.self, from: data)

                return SelfSignedPublicSigningKey<TrustedOrganization>(
                    key: Sign.KeyPair.PublicKey(keyData.key.bytes),
                    certificate: Signature<TrustedOrganization>.fromBytes(bytes: keyData.certificate.bytes),
                    notValidAfter: keyData.notValidAfter.date, now: now
                )
            }
            return nil
        }

        return keys
    }
}
