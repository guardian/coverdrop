import Foundation
import Sodium

public enum IntegrationTestScenario: String {
    case minimal = "minimal_scenario"
    case minimalLegacy = "minimal_scenario_legacy"
    case messaging = "messaging_scenario"
    case multipleJournalists = "multiple_journalists_messaging_scenario"
}

enum IntegrationTestError: Error {
    case cannotFindFile
    case notYetImplemented
    case badDate
}

public class IntegrationTestScenarioContext {
    let config: CoverDropConfig
    let scenario: IntegrationTestScenario

    public init(scenario: IntegrationTestScenario, config: CoverDropConfig = StaticConfig.devConfig) {
        self.config = config
        self.scenario = scenario
    }

    public func getPublicDataRepositoryWithVerifiedKeys(step: String = "001_default") throws -> PublicDataRepository {
        let publicDataRepository = PublicDataRepository(config, urlSession: getMockedUrlSession())
        try publicDataRepository.injectVerifiedPublicKeysForTesting(verifiedPublicKeys: loadKeysVerified(step: step))
        return publicDataRepository
    }

    public func getLibraryWithVerifiedKeys(step: String = "001_default") throws -> CoverDropLibrary {
        let publicDataRepository = try getPublicDataRepositoryWithVerifiedKeys(step: step)
        let secretDataRepository = SecretDataRepository(publicDataRepository: publicDataRepository)
        return CoverDropLibrary(
            publicDataRepository: publicDataRepository,
            secretDataRepository: secretDataRepository,
            config: config
        )
    }

    public func loadKeys(step: String = "001_default") throws -> PublicKeysData {
        guard let resourceUrl = Bundle.module.url(
            forResource: step,
            withExtension: ".json",
            subdirectory: "vectors/\(scenario.rawValue)/published_keys"
        ) else { throw IntegrationTestError.cannotFindFile }
        let data = try Data(contentsOf: resourceUrl)
        let keys = try JSONDecoder().decode(PublicKeysData.self, from: data)
        return keys
    }

    public func loadKeysVerified(step: String = "001_default") throws -> VerifiedPublicKeys {
        let publicKeysData = try loadKeys(step: step)
        let now = try readGeneratedAtFile()
        let trustedOrganizationKeys = try readTrustedOrganizationKeys(now: now)
        return VerifiedPublicKeys(
            publicKeysData: publicKeysData,
            trustedOrganizationPublicKeys: trustedOrganizationKeys,
            currentTime: DateFunction.currentKeysPublishedTime()
        )
    }

    public func loadDeadDrop(step: String = "001_default") throws -> DeadDropData {
        guard let resourceUrl = Bundle.module.url(
            forResource: step,
            withExtension: ".json",
            subdirectory: "vectors/\(scenario.rawValue)/user_dead_drops"
        ) else { throw IntegrationTestError.cannotFindFile }
        let data = try Data(contentsOf: resourceUrl)
        let deadDropData = try JSONDecoder().decode(DeadDropData.self, from: data)
        return deadDropData
    }

    func readTrustedOrganizationKeys(now: Date) throws -> [TrustedOrganizationPublicKey] {
        let resourcePaths: [String] = Bundle.module.paths(
            forResourcesOfType: "json",
            inDirectory: "organization_keys/dev/"
        )

        let keys: [TrustedOrganizationPublicKey] = try resourcePaths.compactMap { fullPath in
            // As `Bundle.module.paths` returns the full path, we just want to get the filename
            let fileName = URL(fileURLWithPath: fullPath).lastPathComponent
            let fileNameWithoutExtension = (fileName as NSString).deletingPathExtension
            let resourceUrlOption = Bundle.module.url(
                forResource: fileNameWithoutExtension,
                withExtension: ".json",
                subdirectory: "organization_keys/dev/"
            )
            if let resourceUrl = resourceUrlOption {
                let data = try Data(contentsOf: resourceUrl)
                let keyData = try JSONDecoder().decode(UnverifiedSignedPublicSigningKeyData.self, from: data)

                return SelfSignedPublicSigningKey<TrustedOrganization>(
                    key: Sign.KeyPair.PublicKey(keyData.key.bytes),
                    certificate: Signature<TrustedOrganization>.fromBytes(bytes: keyData.certificate.bytes),
                    notValidAfter: keyData.notValidAfter.date,
                    now: now
                )
            }
            return nil
        }

        return keys
    }

    public func readGeneratedAtFile() throws -> Date {
        let name = "keys_generated_at"
        guard let resourceUrl = Bundle.module.url(forResource: name, withExtension: ".txt", subdirectory: "keys") else {
            throw IntegrationTestError.cannotFindFile
        }
        let data = try Data(contentsOf: resourceUrl)

        guard let dateString = String(data: data, encoding: .utf8) else {
            throw IntegrationTestError.badDate
        }
        return DateFormats.validateDate(date: dateString)!
    }

    private func getMockedUrlSession() -> URLSession {
        let config = URLSessionConfiguration.ephemeral
        URLProtocolMock.mockURLs = MockUrlData.getMockUrlData()
        config.protocolClasses = [URLProtocolMock.self]
        return URLSession(configuration: config)
    }
}
