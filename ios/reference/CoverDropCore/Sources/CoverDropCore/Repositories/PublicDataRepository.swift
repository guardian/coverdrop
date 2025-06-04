import Foundation
import Sodium

enum PublicDataRepositoryError: Error {
    case configNotAvailable
    case failedToGenerateRandomBytes
    case failedToGetCoverNodeMessageKeys
    case failedToCreateCoverMessage
    case failedToLoadDeadDrops
    case failedToLoadPublicKeys
    case failedToGetConfig
    case notYetImplemented
}

public protocol PublicDataRepositoryProtocol {
    func loadStatus() async throws
    func loadDeadDrops() async throws -> VerifiedDeadDrops
    func pollPublicKeysAndStatusApis() async throws
    func getCoverMessageFactory() throws -> CoverMessageFactory
    func getVerifiedKeys() throws -> VerifiedPublicKeys
    func trySendMessageAndDequeue(_ coverMessageFactory: CoverMessageFactory) async
        -> Result<Int, UserToJournalistMessagingError>
    func getDebugContext() async throws -> DebugContext
}

public typealias CoverMessageFactory = () throws -> MultiAnonymousBox<UserToCoverNodeMessageData>

public class PublicDataRepository: ObservableObject, PublicDataRepositoryProtocol {
    @MainActor @Published public var coverDropServiceStatus: StatusData?
    @MainActor @Published public var areKeysAvailable: Bool = false
    @Published public var cacheEnabled: Bool = true

    public private(set) var config: CoverDropConfig
    private var urlSession: URLSession

    private var verifiedPublicKeys: VerifiedPublicKeys? = .none

    init(_ config: CoverDropConfig, urlSession: URLSession) {
        self.config = config
        self.urlSession = urlSession
    }

    public func pollPublicKeysAndStatusApis() async throws {
        async let status: () = loadStatus()
        async let publicKeys = loadAndVerifyPublicKeys()

        try await status
        _ = try await publicKeys
    }

    public func loadStatus() async throws {
        if let currentStatus = try? await StatusRepository(config: config, urlSession: urlSession)
            .downloadAndUpdateAllCaches(cacheEnabled: config.cacheEnabled) {
            await MainActor.run {
                coverDropServiceStatus = currentStatus
            }
        }
    }

    public func loadDeadDrops() async throws -> VerifiedDeadDrops {
        let deadDropsOpt = try await DeadDropRepository(config: config, urlSession: urlSession)
            .downloadAndUpdateAllCaches(cacheEnabled: config.cacheEnabled)

        guard let deadDrops = deadDropsOpt else {
            throw PublicDataRepositoryError.failedToLoadDeadDrops
        }

        // Load dead drops from journalists
        let verifiedDeadDropData = try VerifiedDeadDrops.fromAllDeadDropData(
            deadDrops: deadDrops,
            verifiedKeys: getVerifiedKeys()
        )

        return verifiedDeadDropData
    }

    /// This loads and verifies the public key and dead drops from the API.
    /// Once verified, they are added to the `publicData` thus available throughtout the app.
    /// Public keys and dead drops can be updated at any time in the API, so we poll to stay up to date.
    public func loadAndVerifyPublicKeys() async throws -> VerifiedPublicKeys {
        let currentKeysPublishedTime = DateFunction.currentKeysPublishedTime()

        // Load public keys
        let publicKeysDataOpt = try? await PublicKeyRepository(
            config: config,
            urlSession: urlSession
        ).downloadAndUpdateAllCaches(cacheEnabled: config.cacheEnabled)

        let trustedRootKeysOpt = try? loadTrustedOrganizationPublicKeys(
            envType: config.envType,
            now: currentKeysPublishedTime
        )

        guard let publicKeysData = publicKeysDataOpt,
              let trustedRootKeys = trustedRootKeysOpt else {
            throw PublicDataRepositoryError.failedToLoadPublicKeys
        }

        let verifiedPublicKeysData = VerifiedPublicKeys(
            publicKeysData: publicKeysData,
            trustedOrganizationPublicKeys: trustedRootKeys,
            currentTime: currentKeysPublishedTime
        )

        await MainActor.run {
            areKeysAvailable = true
            verifiedPublicKeys = verifiedPublicKeysData
        }
        return verifiedPublicKeysData
    }

    func injectVerifiedPublicKeysForTesting(verifiedPublicKeys: VerifiedPublicKeys) {
        self.verifiedPublicKeys = verifiedPublicKeys
    }

    public func sendMessage(message: MultiAnonymousBox<UserToCoverNodeMessageData>) async throws -> HTTPURLResponse {
        let dataOpt = message.asBytes().base64Encode()
        guard let data = dataOpt,
              let jsonData: Data = try? JSONEncoder().encode(data) else {
            throw UserToJournalistMessagingError.unableToBase64Encode
        }

        return try await UserToJournalistMessageWebRepository(
            urlSession: urlSession,
            baseUrl: config.messageBaseUrl
        ).sendMessage(jsonData: jsonData)
    }

    /// This dequeues a message from the `PrivateSendingQueue` and sends it to the user to journalist
    /// message api
    /// 1. dequeue message from privateSendingQueue
    /// 2. send to the api
    public func trySendMessageAndDequeue(_ coverMessageFactory: CoverMessageFactory) async
        -> Result<Int, UserToJournalistMessagingError> {
        let privateSendingQueue = PrivateSendingQueueRepository.shared

        guard let message = try? await privateSendingQueue.peek() else {
            return .failure(UserToJournalistMessagingError.failedToPeekMessage)
        }

        guard let sendResult = try? await sendMessage(message: message) else {
            return .failure(UserToJournalistMessagingError.failedToSendMessage)
        }

        guard (try? await privateSendingQueue.dequeue(coverMessageFactory)) != nil else {
            return .failure(UserToJournalistMessagingError.failedToDequeue)
        }
        return .success(sendResult.statusCode)
    }

    private func createCoverMessageToCoverNode(coverNodeKeys: [PublicEncryptionKey<CoverNodeMessaging>]) throws
        -> MultiAnonymousBox<UserToCoverNodeMessageData> {
        // create placeholder string instead of inner message
        guard let innerEncryptedPlaceholder = Sodium().randomBytes
            .buf(length: Constants.userToJournalistEncryptedMessageLen) else {
            throw PublicDataRepositoryError.failedToGenerateRandomBytes
        }
        if innerEncryptedPlaceholder.count != Constants.userToJournalistEncryptedMessageLen {
            Debug.println("Output length of innerEncryptedPlaceholder was incorrect")
            throw MultiAnonymousBoxError.badOutputLength
        }

        let coverTrafficRecipientTag = Array(repeating: UInt8(0x00), count: Constants.recipientTagLen)
        // build payload of the outer message (to be read by the CoverNode after decryption)
        let payloadForOuter = coverTrafficRecipientTag + innerEncryptedPlaceholder

        if payloadForOuter.count != Constants.userToCovernodeMessageLen {
            throw MultiAnonymousBoxError.badOutputLength
        }

        // encrypt outer message to CoverNode
        let outerEncryptedMessage = try MultiAnonymousBox<UserToCoverNodeMessageData>.encrypt(
            recipientPks: coverNodeKeys,
            data: payloadForOuter
        )
        return outerEncryptedMessage
    }

    public func getCoverMessageFactory() throws -> CoverMessageFactory {
        let verifiedPublicKeys = try getVerifiedKeys()
        let allCoverNodes = verifiedPublicKeys.mostRecentCoverNodeMessagingKeysFromAllHierarchies()
        if allCoverNodes.isEmpty {
            throw PublicDataRepositoryError.failedToGetCoverNodeMessageKeys
        }
        let coverNodeKeys = UserToCoverNodeMessage.selectCoverNodeKeys(coverNodeKeys: allCoverNodes)
        return {
            try self.createCoverMessageToCoverNode(coverNodeKeys: coverNodeKeys)
        }
    }

    public func loadTrustedOrganizationPublicKeys(
        envType: EnvType,
        now: Date
    ) throws -> [TrustedOrganizationPublicKey] {
        try OrganizationKeysLoader.loadTrustedOrganizationPublicKeys(envType: envType, now: now)
    }

    public func getVerifiedKeys() throws -> VerifiedPublicKeys {
        guard let verifiedKeys = verifiedPublicKeys else {
            throw PublicDataRepositoryError.failedToLoadPublicKeys
        }
        return verifiedKeys
    }

    public func getDebugContext() async throws -> DebugContext {
        let orgKeys = try? loadTrustedOrganizationPublicKeys(
            envType: config.envType,
            now: DateFunction.currentTime()
        )
        let keyDigests = orgKeys?.map { getHumanReadableDigest(key: $0.key) }
        let maybeKeyDigestsString = keyDigests?.joinTo(separator: "; ", prefix: "[", suffix: "]")

        let lastUpdatePublicKeys = try? await PublicKeyRepository(
            config: config,
            urlSession: urlSession
        ).getTimestampOfCachedFile()
        let lastUpdateDeadDrops = try? await DeadDropRepository(
            config: config,
            urlSession: urlSession
        ).getTimestampOfCachedFile()

        let lastBackgroundTry = BackgroundMessageSendState.readBackgroundWorkLastTrigger()
        let lastBackgroundSend = BackgroundMessageSendState.readBackgroundWorkLastSuccessfulRun()

        return DebugContext(
            lastUpdatePublicKeys: lastUpdatePublicKeys,
            lastUpdateDeadDrops: lastUpdateDeadDrops,
            lastBackgroundTry: lastBackgroundTry,
            lastBackgroundSend: lastBackgroundSend,
            hashedOrgKeys: maybeKeyDigestsString
        )
    }
}
