@testable import CoverDropCore
import Sodium
import XCTest

final class PrivateSendingQueueTests: XCTestCase {
    private let secret = PrivateSendingQueueSecret(bytes: "secret__secret__".asBytes())
    // swiftlint:disable:next force_try
    private let publicDataRepository: PublicDataRepository = try! getPublicDataRepository()

    static func getPublicDataRepository() throws -> PublicDataRepository {
        let context = IntegrationTestScenarioContext(scenario: .minimal)
        return try context.getPublicDataRepositoryWithVerifiedKeys()
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

    private func emptyCoverdropQueue() throws -> PrivateSendingQueue {
        guard let coverMessage = try? publicDataRepository.getCoverMessageFactory() else {
            XCTFail("Failed to get cover message")
            throw PublicDataRepositoryError.failedToCreateCoverMessage
        }
        return try PrivateSendingQueue(
            totalQueueSize: PrivateSendingQueueConfiguration.default.totalQueueSize,
            messageSize: PrivateSendingQueueConfiguration.default.messageSize,
            coverMessageFactory: coverMessage
        )
    }

    func testEnqueueWhenAddingMessageThenFillLevelIncreases() async throws {
        var queue = try emptyCoverdropQueue()
        XCTAssertTrue(queue.getFillLevel(secret: secret!) == 0)

        _ = try await queue.enqueue(secret: secret!, message: message("message1"))
        XCTAssertTrue(queue.getFillLevel(secret: secret!) == 1)

        _ = try await queue.enqueue(secret: secret!, message: message("message2"))
        XCTAssertTrue(queue.getFillLevel(secret: secret!) == 2)
    }

    func testEnqueueWhenCheckingFillLevelWithWrongSecretThenReturnsEmpty() async throws {
        let wrongSecret = PrivateSendingQueueSecret(bytes: "__terces__terces".asBytes())

        var queue = try emptyCoverdropQueue()
        XCTAssertTrue(queue.getFillLevel(secret: secret!) == 0)

        _ = try await queue.enqueue(secret: secret!, message: message("message1"))
        XCTAssertTrue(queue.getFillLevel(secret: wrongSecret!) == 0)

        _ = try await queue.enqueue(secret: secret!, message: message("message2"))
        XCTAssertTrue(queue.getFillLevel(secret: wrongSecret!) == 0)
    }

    func testEnqueueWhenAddingMessagesBeyondCapacityThenSpaceThenThrows() async throws {
        let message1 = try await message("message1")
        var queue = try emptyCoverdropQueue()
        for _ in 0 ..< PrivateSendingQueueConfiguration.default.totalQueueSize {
            _ = try queue.enqueue(secret: secret!, message: message1)
        }

        XCTAssertThrowsError(_ = try queue.enqueue(secret: secret!, message: message1)) { error in
            XCTAssertEqual(error as! PrivateSendingQueueError, PrivateSendingQueueError.queueIsFull)
        }
    }

    func testEnqueueWhenAddingMessageWithDifferentSecretThenOthersOverwritten() async throws {
        let message1 = try await message("message1")
        let message2 = try await message("message2")
        let message3 = try await message("message3")

        let differentSecret = PrivateSendingQueueSecret(bytes: "__terces__terces".asBytes())

        var queue = try emptyCoverdropQueue()
        _ = try queue.enqueue(secret: secret!, message: message1)
        _ = try queue.enqueue(secret: secret!, message: message2)
        _ = try queue.enqueue(secret: differentSecret!, message: message3)

        // we would have otherwise expected message1 due to the FIFO nature of the queue
        XCTAssertEqual(
            try queue.sendHeadMessageAndPushNewCoverMessage(publicDataRepository.getCoverMessageFactory()),
            message3
        )
    }

    func testDequeueWhenAddedMessagesThenPoppedInOrder() async throws {
        let message1 = try await message("message1")
        let message2 = try await message("message2")

        var queue = try emptyCoverdropQueue()
        _ = try queue.enqueue(secret: secret!, message: message1)
        _ = try queue.enqueue(secret: secret!, message: message2)

        XCTAssertEqual(
            try queue.sendHeadMessageAndPushNewCoverMessage(publicDataRepository.getCoverMessageFactory()),
            message1
        )
        XCTAssertEqual(
            try queue.sendHeadMessageAndPushNewCoverMessage(publicDataRepository.getCoverMessageFactory()),
            message2
        )
    }

    func testDequeueWhenPoppingMoreThanRealMessagesThenCoverMessagesReturned() async throws {
        let message1 = try await message("message1")
        let message2 = try await message("message2")

        var queue = try emptyCoverdropQueue()
        _ = try queue.enqueue(secret: secret!, message: message1)
        _ = try queue.enqueue(secret: secret!, message: message2)

        XCTAssertEqual(
            try queue.sendHeadMessageAndPushNewCoverMessage(publicDataRepository.getCoverMessageFactory()),
            message1
        )
        XCTAssertEqual(
            try queue.sendHeadMessageAndPushNewCoverMessage(publicDataRepository.getCoverMessageFactory()),
            message2
        )

        _ = try queue.sendHeadMessageAndPushNewCoverMessage(publicDataRepository.getCoverMessageFactory())

        XCTAssertNotEqual(
            try queue.sendHeadMessageAndPushNewCoverMessage(publicDataRepository.getCoverMessageFactory()),
            message1
        )
        XCTAssertNotEqual(
            try queue.sendHeadMessageAndPushNewCoverMessage(publicDataRepository.getCoverMessageFactory()),
            message2
        )
    }

    func testFromByteFailsOnEmptyByteArray() throws {
        XCTAssertThrowsError(try PrivateSendingQueue.fromBytes(bytes: []))
    }

    func testFromByteFailsOnTruncatedByteArray() throws {
        XCTAssertThrowsError(try PrivateSendingQueue.fromBytes(bytes: [] + Data([0x01])))
    }

    func testFromBytesWhenSerdeEmptyThenDeserializesSuccessfully() throws {
        let original = try emptyCoverdropQueue()
        let serialized = try original.serialize()
        let new = try PrivateSendingQueue.fromBytes(bytes: serialized)
        XCTAssert(original == new)
    }

    func testFromBytesWhenSerdeWithMessagesThenDeserializesSuccessfully() async throws {
        let message1 = try await message("message1")
        let message2 = try await message("message2")

        var original = try emptyCoverdropQueue()
        _ = try original.enqueue(secret: secret!, message: message1)
        _ = try original.enqueue(secret: secret!, message: message2)

        let serialized = try original.serialize()

        var copy = try PrivateSendingQueue.fromBytes(bytes: serialized)

        let fillLevel = copy.getFillLevel(secret: secret!)
        XCTAssertEqual(fillLevel, 2)

        let actualMessage1 = try copy
            .sendHeadMessageAndPushNewCoverMessage(publicDataRepository.getCoverMessageFactory())
        XCTAssertEqual(actualMessage1, message1)

        let actualMessage2 = try copy
            .sendHeadMessageAndPushNewCoverMessage(publicDataRepository.getCoverMessageFactory())
        XCTAssertEqual(actualMessage2, message2)

        try original.sendHeadMessageAndPushNewCoverMessage(publicDataRepository.getCoverMessageFactory()) // message 1
        try original.sendHeadMessageAndPushNewCoverMessage(publicDataRepository.getCoverMessageFactory()) // message 2
        let originalCover1 = try original
            .sendHeadMessageAndPushNewCoverMessage(publicDataRepository.getCoverMessageFactory())
        let actualCover1 = try copy.sendHeadMessageAndPushNewCoverMessage(publicDataRepository.getCoverMessageFactory())
        XCTAssertEqual(originalCover1, actualCover1)
    }

    func testCanCheckMessageStillInQueue() async throws {
        let message1 = try await message("message1")

        var original = try emptyCoverdropQueue()
        let hint1 = try original.enqueue(secret: secret!, message: message1)

        let isInQueue = original.isMessageStillInQueue(hint: hint1)

        XCTAssertTrue(isInQueue)

        try original.sendHeadMessageAndPushNewCoverMessage(publicDataRepository.getCoverMessageFactory()) // message 1
        try original.sendHeadMessageAndPushNewCoverMessage(publicDataRepository.getCoverMessageFactory()) // message 2

        let isStillInQueue = original.isMessageStillInQueue(hint: hint1)

        XCTAssertFalse(isStillInQueue)
    }

    func testAllCoverMessagesAreUnique() async throws {
        let message1 = try await message("message1")
        let message2 = try await message("message2")

        var original = try emptyCoverdropQueue()

        XCTAssertTrue(original.mStorage.count == Set(original.mStorage).count)

        _ = try original.enqueue(secret: secret!, message: message1)
        _ = try original.enqueue(secret: secret!, message: message2)

        XCTAssertTrue(original.mStorage.count == Set(original.mStorage).count)

        try original.sendHeadMessageAndPushNewCoverMessage(publicDataRepository.getCoverMessageFactory()) // message 1
        try original.sendHeadMessageAndPushNewCoverMessage(publicDataRepository.getCoverMessageFactory()) // message 2

        XCTAssertTrue(original.mStorage.count == Set(original.mStorage).count)
    }
}
