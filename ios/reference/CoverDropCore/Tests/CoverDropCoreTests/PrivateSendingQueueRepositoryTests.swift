@testable import CoverDropCore
import Sodium
import XCTest

final class PrivateSendingQueueRepositoryTests: XCTestCase {
    private let secret = PrivateSendingQueueSecret(bytes: "secret__secret__".asBytes())!

    // swiftlint:disable:next force_try
    private let publicDataRepository: PublicDataRepository = try! getPublicDataRepository()

    static func getPublicDataRepository() throws -> PublicDataRepository {
        let context = IntegrationTestScenarioContext(scenario: .minimal)
        return try context.getPublicDataRepositoryWithVerifiedKeys()
    }

    private func getJournalistMessagingKey() throws -> JournalistMessagingPublicKey {
        let verifiedKeys = try publicDataRepository.getVerifiedKeys()
        return verifiedKeys.allMessageKeysForJournalistId(journalistId: "static_test_journalist").first!
    }

    override func setUp() async throws {
        try StorageManager.shared.deleteFile(file: CoverDropFiles.privateSendingQueueV2)
    }

    private func message(_ message: String) async throws -> MultiAnonymousBox<UserToCoverNodeMessageData> {
        let userKeyPair: EncryptionKeypair<User> = try EncryptionKeypair<User>.generateEncryptionKeypair()
        let recipientPublicKey = try publicDataRepository
            .getVerifiedKeys().getLatestMessagingKey(journalistId: "static_test_journalist")!

        let encryptedMessage = try UserToCoverNodeMessage.createMessage(
            message: message,
            recipientPublicKey: recipientPublicKey,
            verifiedPublicKeys: publicDataRepository.getVerifiedKeys(),
            userPublicKey: userKeyPair.publicKey,
            tag: RecipientTag(tag: [2, 3, 3, 3])
        )
        return encryptedMessage
    }

    func testRoundTrip() async throws {
        let secret = PrivateSendingQueueSecret(bytes: "secret__secret__".asBytes())

        let userKeyPair: EncryptionKeypair<User> = try EncryptionKeypair<User>.generateEncryptionKeypair()
        let encryptedMessage = try UserToCoverNodeMessage.createMessage(
            message: "message 1",
            recipientPublicKey: getJournalistMessagingKey(),
            verifiedPublicKeys: publicDataRepository.getVerifiedKeys(),
            userPublicKey: userKeyPair.publicKey,
            tag: RecipientTag(tag: [1, 2, 3, 4])
        )

        let testableRepo = PrivateSendingQueueRepository.shared
        let initialQueue = try await testableRepo.loadOrInitialiseQueue(publicDataRepository.getCoverMessageFactory())

        _ = try await testableRepo.enqueue(secret: secret!, message: encryptedMessage)

        let newQueueState = try await testableRepo.loadOrInitialiseQueue(publicDataRepository.getCoverMessageFactory())

        XCTAssert(initialQueue != newQueueState)
    }

    func testAddingTwoMessagesWithinCapacity() async throws {
        let testableRepo = PrivateSendingQueueRepository.shared
        _ = try await testableRepo.loadOrInitialiseQueue(publicDataRepository.getCoverMessageFactory())

        // GIVEN a queue with 2 empty slots
        for _ in 0 ..< (PrivateSendingQueueConfiguration.default.totalQueueSize - 2) {
            _ = try await testableRepo.enqueue(secret: secret, message: await message("message1"))
        }

        // WHEN attempting to add 2 messages
        let errorsCount = try await addTwoMessagesConcurrentlyReturningErrorCount(to: testableRepo)

        // THEN 0 errors should be thrown
        XCTAssertEqual(errorsCount, 0)
    }

    func testAddingOneMessageBeyondCapacity() async throws {
        let testableRepo = PrivateSendingQueueRepository.shared
        _ = try await testableRepo.loadOrInitialiseQueue(publicDataRepository.getCoverMessageFactory())

        let message = try await message("message1")

        // GIVEN a queue with 1 empty slots
        for _ in 0 ..< (PrivateSendingQueueConfiguration.default.totalQueueSize - 1) {
            _ = try await testableRepo.enqueue(secret: secret, message: message)
        }

        // WHEN attempting to add 2 messages
        let errorsCount = try await addTwoMessagesConcurrentlyReturningErrorCount(to: testableRepo)

        // THEN 1 errors should be thrown
        XCTAssertEqual(errorsCount, 1)
    }

    func testAddingTwoMessagesBeyondCapacity() async throws {
        let testableRepo = PrivateSendingQueueRepository.shared
        _ = try await testableRepo.loadOrInitialiseQueue(publicDataRepository.getCoverMessageFactory())

        let message = try await message("message1")

        // GIVEN a queue with 0 empty slots
        for _ in 0 ..< PrivateSendingQueueConfiguration.default.totalQueueSize {
            _ = try await testableRepo.enqueue(secret: secret, message: message)
        }

        // WHEN attempting to add 2 messages
        let errorsCount = try await addTwoMessagesConcurrentlyReturningErrorCount(to: testableRepo)

        // THEN 2 errors should be thrown
        XCTAssertEqual(errorsCount, 2)
    }

    @discardableResult
    private func addTwoMessagesConcurrentlyReturningErrorCount(
        to repository: PrivateSendingQueueRepository
    ) async throws -> Int {
        let taskPriorities: [TaskPriority] = [.low, .high, .medium, .utility, .background, .userInitiated]

        let message = try await message("message1")

        let addTask = Task.detached(priority: taskPriorities.randomElement()) {
            try await repository.enqueue(secret: self.secret, message: message)
        }

        let addTask2 = Task.detached(priority: taskPriorities.randomElement()) {
            try await repository.enqueue(secret: self.secret, message: message)
        }

        var errorsCount = 0

        do {
            _ = try await addTask.result.get()
        } catch {
            errorsCount += 1
        }

        do {
            _ = try await addTask2.result.get()
        } catch {
            errorsCount += 1
        }

        return errorsCount
    }

    func testEnqueued() async throws {
        let testableRepo = PrivateSendingQueueRepository.shared
        let coverMessageFactory = try publicDataRepository.getCoverMessageFactory()
        let initialQueue = try await testableRepo.loadOrInitialiseQueue(publicDataRepository.getCoverMessageFactory())

        // THEN the queue should be empty initially
        let fillLevel = initialQueue.getFillLevel(secret: secret)
        XCTAssertEqual(fillLevel, 0)

        // WHEN two messages are added
        try await addTwoMessagesConcurrentlyReturningErrorCount(to: testableRepo)

        // THEN the repo's queue should have 2 messages in memory
        let filled = try await testableRepo.loadOrInitialiseQueue(coverMessageFactory)
        let fillLevel2 = filled.getFillLevel(secret: secret)
        XCTAssertEqual(fillLevel2, 2)
    }

    func testDequeue() async throws {
        let testableRepo = PrivateSendingQueueRepository.shared
        let coverMessageFactory = try publicDataRepository.getCoverMessageFactory()
        _ = try await testableRepo.loadOrInitialiseQueue(coverMessageFactory)

        try await addTwoMessagesConcurrentlyReturningErrorCount(to: testableRepo)

        // WHEN dequeue is called
        _ = try await testableRepo.dequeue(coverMessageFactory)

        // THEN the queue should have 1 message left
        let queue = try await testableRepo.loadOrInitialiseQueue(coverMessageFactory)
        let fillLevel = queue.getFillLevel(secret: secret)
        XCTAssertEqual(fillLevel, 1)
    }

    func testPeek() async throws {
        let testableRepo = PrivateSendingQueueRepository.shared
        let coverMessageFactory = try publicDataRepository.getCoverMessageFactory()
        _ = try await testableRepo.loadOrInitialiseQueue(coverMessageFactory)
        try await addTwoMessagesConcurrentlyReturningErrorCount(to: testableRepo)

        // WHEN peek is called
        _ = try await testableRepo.peek()
        // THEN the queue should still have 2 message left
        let queue = try await testableRepo.loadOrInitialiseQueue(coverMessageFactory)
        let fillLevel = queue.getFillLevel(secret: secret)
        XCTAssertEqual(fillLevel, 2)
    }

    func testWipe() async throws {
        let secret = PrivateSendingQueueSecret(bytes: "secret__secret__".asBytes())
        let encryptedMessage = try await message("message1")

        // GIVEN an initialized empty queue
        let coverMessageFactory = try publicDataRepository.getCoverMessageFactory()
        let testableRepo = PrivateSendingQueueRepository.shared
        _ = try await testableRepo.loadOrInitialiseQueue(coverMessageFactory)

        // WHEN a message is enqueue, then the queue is wiped
        let hmac = try await testableRepo.enqueue(secret: secret!, message: encryptedMessage)

        let inQueue = try await testableRepo.isMessageInQueue(hint: hmac)

        XCTAssertTrue(inQueue)

        try await testableRepo.wipeQueue(coverMessageFactory)
        // THEN a message is no longer in the queue
        let stillInQueue = try await testableRepo.isMessageInQueue(hint: hmac)

        XCTAssertFalse(stillInQueue)
    }

    func testSaving() async throws {
        let coverMessageFactory = try publicDataRepository.getCoverMessageFactory()

        // GIVEN a PrivateSendingQueueDataStore and PrivateSendingQueue are initialized
        let sut = PrivateSendingQueueRepository.shared
        let defaultConfig = PrivateSendingQueueConfiguration.default
        let queue = try PrivateSendingQueue(
            totalQueueSize: defaultConfig.totalQueueSize,
            messageSize: defaultConfig.messageSize,
            coverMessageFactory: coverMessageFactory
        )

        // WHEN that queue is saved to the store
        try await sut.saveQueue(queue)

        // THEN if the queue is later loaded from disk, it is equivalent to the queue in memory
        let fileContents = try StorageManager.shared.readFile(file: CoverDropFiles.privateSendingQueueV2)
        let newQueue = try PrivateSendingQueue.fromBytes(bytes: fileContents)
        XCTAssert(newQueue == queue)
    }

    func testLoading() async throws {
        let coverMessageFactory = try publicDataRepository.getCoverMessageFactory()

        // GIVEN a PrivateSendingQueueDataStore and PrivateSendingQueue are initialized,
        // and a message is added to the queue, and then saved to disk
        let sut = PrivateSendingQueueRepository.shared
        let defaultConfig = PrivateSendingQueueConfiguration.default
        var queue = try PrivateSendingQueue(totalQueueSize: defaultConfig.totalQueueSize,
                                            messageSize: defaultConfig.messageSize,
                                            coverMessageFactory: coverMessageFactory)
        let message = try await message("message1")
        _ = try queue.enqueue(secret: secret, message: message)
        try await sut.saveQueue(queue)

        // WHEN that queue is loaded from disk
        let newQueue = try await sut.loadQueue()

        // THEN that queue is equivalent to the queue in memory
        XCTAssert(newQueue == queue)

        // AND the queue remains on disk after being loaded once (do the same action again)
        let newestQueue = try await sut.loadQueue()
        XCTAssert(newestQueue == queue)
    }
}
