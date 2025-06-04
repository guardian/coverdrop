import CryptoKit
import Foundation
import Sodium

enum SecretDataRepositoryError: Error {
    case secretDataIsLocked
    case repositoriesNotAvailable
}

public protocol SecretDataRepositoryProtocol {
    /// Callback whenever the app starts or CoverDrop is initialized for the first time.
    func onAppStart() async throws

    /// Callback whenever the app enters background.
    func onDidEnterBackground() async throws

    /// Returns the current secret data state (might be unlocked or locked)
    func getSecretData() -> SecretData

    /// Creates or resets the vault with the given passphrase.
    func createOrReset(passphrase: ValidPassword) async throws

    /// Deletes the vault and resets the passphrase to a randomly generated one.
    func deleteVault() async throws

    /// Locks the vault and clears the secret data.
    func lock() async throws

    /// Unlocks the vault with the given passphrase. Throws if the passphrase is incorrect.
    func unlock(passphrase: ValidPassword) async throws

    /// Returns all mailbox recipients (journalists or desks) that the user has had conversations with.
    func getMailboxRecipients(publicKeyData: VerifiedPublicKeys) async throws -> [JournalistData]

    /// Adds a new message to the mailbox (will automatically sorted to the right conversation).
    func addMessage(message: Message) async throws

    /// Adds a new messages to the mailbox (will automatically sorted to the right conversation).
    func addMessages(messages: Set<Message>) async throws

    /// Sends a message to a recipient. This will encrypt the message, add it to the PSQ, and add it to the mailbox.
    func sendMessage(
        _ message: String,
        to recipient: JournalistData,
        dateSent: Date
    ) async throws

    func setUnlockedDataForTesting(unlockedData: UnlockedSecretData)
}

public class SecretDataRepository: ObservableObject, SecretDataRepositoryProtocol {
    @Published public var secretData: SecretData = .lockedSecretData(lockedData: LockedSecretData())
    private var publicDataRepository: PublicDataRepository
    private var encryptedStorage: EncryptedStorage = EncryptedStorage.createForSecretDataRepository()

    init(publicDataRepository: PublicDataRepository) {
        self.publicDataRepository = publicDataRepository
    }

    private var encryptedStorageSession: EncryptedStorageSession?

    public func onAppStart() async throws {
        try await encryptedStorage.onAppStart(config: publicDataRepository.config)
    }

    public func onDidEnterBackground() async throws {
        try encryptedStorage.onDidEnterBackground()
    }

    public func getSecretData() -> SecretData {
        return secretData
    }

    public func createOrReset(passphrase: ValidPassword) async throws {
        encryptedStorageSession = try await encryptedStorage.createOrResetStorageWithPassphrase(passphrase: passphrase)
        try await loadData()
    }

    /// Deletes the vault and resets the passphrase to a randomly generated one. This will also
    /// wipe the private sending queue to remove any pending messages. Afterwards, the
    /// session will be set to a locked state.
    public func deleteVault() async throws {
        let passphrase = PasswordGenerator.shared.generate(wordCount: publicDataRepository.config.passphraseWordCount)
        try await createOrReset(passphrase: passphrase)

        if let coverMessageFactory = try? publicDataRepository.getCoverMessageFactory() {
            try await PrivateSendingQueueRepository.shared.wipeQueue(coverMessageFactory)
        }

        await MainActor.run {
            secretData = .lockedSecretData(lockedData: LockedSecretData())
        }
    }

    public func unlock(passphrase: ValidPassword) async throws {
        // unlock session and use it to load the inital data
        encryptedStorageSession = try await encryptedStorage.unlockStorageWithPassphrase(passphrase: passphrase)
        try await loadData()
        try await decryptDeadDrops()
    }

    private func decryptDeadDrops() async throws {
        try await DeadDropDecryptionService().decryptStoredDeadDrops(
            publicDataRepository: publicDataRepository,
            secretDataRepository: self
        )
    }

    private func loadData() async throws {
        let unlockedData = try await encryptedStorage.loadStorageFromDisk(session: encryptedStorageSession!)
        await MainActor.run {
            secretData = .unlockedSecretData(unlockedData: unlockedData)
        }
    }

    public func lock() async throws {
        // Note that we expire old messages when we save the mailbox content, and not when unlocking. This ensures that
        // messages we just received, but which might be very old based on their dead drop timestamp, are displayed at
        // least once.
        try await expireOldMessages()
        try await storeData()
        await MainActor.run {
            secretData = .lockedSecretData(lockedData: LockedSecretData())
        }
    }

    public func sendMessage(
        _ message: String,
        to recipient: JournalistData,
        dateSent: Date
    ) async throws {
        guard case let .unlockedSecretData(unlockedData: unlockedData) = secretData else {
            throw SecretDataRepositoryError.secretDataIsLocked
        }

        guard let lib = try? CoverDropService.getLibrary() else {
            throw SecretDataRepositoryError.repositoriesNotAvailable
        }

        let encryptedMessage = try await UserToCoverNodeMessageData.createMessage(
            message: message,
            messageRecipient: recipient,
            verifiedPublicKeys: lib.publicDataRepository.getVerifiedKeys(),
            userPublicKey: unlockedData.userKey.publicKey
        )

        let hint = try await PrivateSendingQueueRepository.shared.enqueue(
            secret: unlockedData.privateSendingQueueSecret,
            message: encryptedMessage
        )

        let outboundMessage = OutboundMessageData(
            recipient: recipient,
            messageText: message,
            dateQueued: dateSent,
            hint: hint
        )

        let newMessage: Message = .outboundMessage(message: outboundMessage)
        try await addMessage(message: newMessage)
    }

    public func addMessage(message: Message) async throws {
        if case let .unlockedSecretData(unlockedData: unlockedData) = secretData {
            await unlockedData.addMessage(message: message)
            try await storeData()
        }
    }

    public func addMessages(messages: Set<Message>) async throws {
        if case let .unlockedSecretData(unlockedData: unlockedData) = secretData {
            await unlockedData.addMessages(messages: messages)
            try await storeData()
        }
    }

    public func expireOldMessages() async throws {
        if case let .unlockedSecretData(unlockedData: unlockedData) = secretData {
            let now = DateFunction.currentTime()
            let cutoff = try now.minusSeconds(Constants.messageValidForDurationInSeconds)
            await unlockedData.removeExpiredMessages(cutoff: cutoff)
            try await storeData()
        }
    }

    public func storeData() async throws {
        if case let .unlockedSecretData(unlockedData: unlockedData) = secretData {
            try await encryptedStorage.updateStorageOnDisk(
                session: encryptedStorageSession!,
                state: unlockedData
            )
        }
    }

    /// This gets the journalist or desks that the user has had converstations with
    /// This is used when we decrypt incoming dead drops so that we only try with keys for journalists
    /// we've been in converstations with.
    /// - Returns: A list of JournalistKeyData
    public func getMailboxRecipients(publicKeyData: VerifiedPublicKeys) async throws -> [JournalistData] {
        guard case let .unlockedSecretData(unlockedData: unlockedData) = secretData else {
            throw SecretDataRepositoryError.secretDataIsLocked
        }
        let recipients = unlockedData.messageMailbox.compactMap { message in
            switch message {
            case let .outboundMessage(message: message):
                return message.recipient
            case let .incomingMessage(message: messageType):
                switch messageType {
                case let .textMessage(message: message):
                    return message.sender
                case let .handoverMessage(message: handover):
                    return publicKeyData.getJournalistKeyDataForJournalistId(
                        journalistId: handover.handoverTo
                    )
                }
            }
        }
        let uniqueRecipients = Set(recipients)
        return Array(uniqueRecipients)
    }

    public func setUnlockedDataForTesting(unlockedData: UnlockedSecretData) {
        secretData = .unlockedSecretData(unlockedData: unlockedData)
    }
}
