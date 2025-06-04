import Foundation

enum DeadDropDecryptionServiceError: Error {
    case failedToGetKeys
    case failedToGetDeadDrops
}

public struct DeadDropDecryptionService {
    public init() {}

    /// This service tries to decrypts the supplied verfied dead drops with the supplied journalist Key
    /// If a message within the dead drops is succesfully decrypted it is added to the user mailbox
    ///
    public func decryptStoredDeadDrops(
        publicDataRepository: any PublicDataRepositoryProtocol,
        secretDataRepository: any SecretDataRepositoryProtocol
    ) async throws {
        guard let verifiedDeadDrops = try? await publicDataRepository.loadDeadDrops() else {
            throw DeadDropDecryptionServiceError.failedToGetDeadDrops
        }

        // we only have access to the user secret key when we are unlocked
        // so this is the only state we can be in to try and decrypt the data
        if case let .unlockedSecretData(unlockedData: secretData) = secretDataRepository.getSecretData() {
            let verifiedPublicKeys = try publicDataRepository.getVerifiedKeys()
            let userSecretKey: SecretEncryptionKey<User> = secretData.userKey.secretKey

            let currentConversationJournalists = try await secretDataRepository
                .getMailboxRecipients(publicKeyData: verifiedPublicKeys)

            var messages: Set<Message> = []
            for journalistData in currentConversationJournalists {
                let message = await DecryptedDeadDrops.decryptWithUserKey(
                    userSecretKey: userSecretKey,
                    journalistData: journalistData,
                    verifiedDeadDropData: verifiedDeadDrops,
                    verifiedPublicKeys: verifiedPublicKeys
                )
                messages.formUnion(message)
            }

            try await secretDataRepository.addMessages(messages: messages)
        }
    }
}
