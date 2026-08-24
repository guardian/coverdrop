@testable import CoverDropCore
import CryptoKit
import Sodium
import XCTest

final class DeadDropDecryptionServiceTests: XCTestCase {
    override func setUp() async throws {
        try super.setUpWithError()
        try StorageManager.shared.deleteFile(file: CoverDropFiles.encryptedStorage)
        TestingBridge.setCurrentTimeOverride(override: CoverDropServiceHelper.currentTimeForKeyVerification())
    }

    func testDeadDropDecryption() async throws {
        let context = IntegrationTestScenarioContext(
            scenario: .messaging,
            config: StaticConfig.devConfigWithCache
        )
        let publicDataRepository = try context.getPublicDataRepositoryWithVerifiedKeys(
            step: "003_journalist_replied_and_processed"
        )
        let secretDataRepository = SecretDataRepository(publicDataRepository: publicDataRepository)

        // Setup the dead drop Id repository to match the Id in the test vector (which is id 1)
        try await DeadDropIdRepository().save(deadDropId: DeadDropId(id: 0))

        // Load deaddrops into cache file
        try await publicDataRepository.fetchDeadDrops()

        // Set up the secrect data repository to use the user keys from the test vector
        let passphrase = ValidPassword(password: "external jersey squeeze luckiness collector")
        try await secretDataRepository.createOrReset(passphrase: passphrase)

        let recipient = try XCTUnwrap(PublicKeysHelper.shared.testDefaultJournalist)

        // add the user test vector keys to the secret data repository, with an empty mailbox
        let userSecretMessageKey = try PublicKeysHelper.shared.getTestUserMessageSecretKey()
        let userPublicMessageKey = try PublicKeysHelper.shared.getTestUserMessagePublicKey()
        let userKeyPair = EncryptionKeypair(publicKey: userPublicMessageKey, secretKey: userSecretMessageKey)
        let privateSendingQueueSecret = try PrivateSendingQueueSecret.fromSecureRandom()

        if case let .unlockedSecretData(data) = secretDataRepository.secretData {
            data.userKey = userKeyPair
            data.privateSendingQueueSecret = privateSendingQueueSecret
        } else {
            XCTFail("not in unlocked data state")
        }

        let outboundMessage = OutboundMessageData(
            recipient: recipient,
            messageText: "Hey",
            dateQueued: DateFunction.currentTime(),
            hint: HintHmac(hint: [1, 2, 3, 4])
        )
        try await secretDataRepository.addMessage(message: .outboundMessage(message: outboundMessage))
        try await secretDataRepository.lock()
        if case .lockedSecretData = secretDataRepository.secretData {
            XCTAssertTrue(true)
        } else {
            XCTFail("secretDataRepository was not locked after locking")
        }

        // unlock the secretDataRepository
        // the internal application state should now have:
        // public keys from test vector
        // user keys from test vector are used on the secrectDataRepository
        // the dead drop test vector contains a message from `static_test_journalist` whos keys are in the public keys

        try await secretDataRepository.unlock(passphrase: passphrase)
        if case let .unlockedSecretData(data) = secretDataRepository.secretData {
            let deadDropData = try DeadDropDataHelper.shared.readLocalDataFile()
            _ = try VerifiedDeadDrops.fromAllDeadDropData(
                deadDrops: deadDropData,
                verifiedKeys: publicDataRepository.getVerifiedKeys()
            )

            // Neither `id` nor the sub-second part of `createdAt` is signed, so these replays all verify.
            var replayedDeadDropData = deadDropData
            replayedDeadDropData.deadDrops += deadDropData.deadDrops.map { deadDrop in
                var replayed = deadDrop
                replayed.id = deadDrop.id + 1000
                replayed.createdAt = RFC3339DateTimeString(
                    date: Date(timeIntervalSince1970: TimeInterval(deadDrop.createdAt.epochSeconds) + 0.999)
                )
                return replayed
            }

            let repo = DeadDropRepository(config: StaticConfig.devConfig, urlSession: URLSession.shared)
            try await repo.localRepository.save(data: replayedDeadDropData)
            try await DeadDropDecryptionService().decryptStoredDeadDrops(
                publicDataRepository: publicDataRepository,
                secretDataRepository: secretDataRepository
            )

            let matchingMessages = data.messageMailbox.filter {
                if case let .incomingMessage(message: message) = $0 {
                    if case let .textMessage(textMessage) = message {
                        return textMessage.messageText == "This is a test message from the journalist to the user"
                    }
                }
                return false
            }

            XCTAssertEqual(matchingMessages.count, 1, "replayed dead drops must not duplicate the message")
        } else {
            XCTFail("not in unlocked data")
        }
    }
}
