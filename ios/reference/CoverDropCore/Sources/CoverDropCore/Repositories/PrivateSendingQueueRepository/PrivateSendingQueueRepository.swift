import Foundation

enum PrivateSendingQueueRepositoryError: Error {
    case queueNotAvailable
}

/// This repository actor is the main interface for the app for creating and managing its instance of the global
/// `PrivateSendingQueue` via a  `shared` singleton instance.
public actor PrivateSendingQueueRepository: ObservableObject {
    public static let shared = PrivateSendingQueueRepository(StorageManager.shared)

    @MainActor @Published public var hintsInFlight: [HintHmac] = []

    private let storageManager: StorageManager
    private let psqFile = CoverDropFiles.privateSendingQueueV2

    private init(_ storageManager: StorageManager) {
        self.storageManager = storageManager
    }

    /// Tries to load the `PrivateSendingQueue` from disk. If it doesn't exist, it will create a new one.
    public func loadOrInitialiseQueue(_ coverMessageFactory: CoverMessageFactory) async throws -> PrivateSendingQueue {
        let configuration = PrivateSendingQueueConfiguration.default
        if let queueFromDisk = try await loadQueue() {
            let psq = queueFromDisk
            try await updateHintsInFlight(psq)
            return psq
        } else {
            let queue = try PrivateSendingQueue(
                totalQueueSize: configuration.totalQueueSize,
                messageSize: configuration.messageSize,
                coverMessageFactory: coverMessageFactory
            )
            try await saveQueue(queue)
            return queue
        }
    }

    /// Removes the current queue, by generating a new `PrivateSendingQueue` and storing to disk
    /// This is so we can remove any pending real messages from users if they choose to delete all messages.
    /// - Parameter configuration: An optional configuration for setting up the managed`PrivateSendingQueue`.
    public func wipeQueue(
        with configuration: PrivateSendingQueueConfiguration = PrivateSendingQueueConfiguration.default,
        _ coverMessageFactory: CoverMessageFactory
    ) async throws {
        let queue = try PrivateSendingQueue(
            totalQueueSize: configuration.totalQueueSize,
            messageSize: configuration.messageSize,
            coverMessageFactory: coverMessageFactory
        )
        try await saveQueue(queue)
    }

    /// Enqueues the given message by calling the the `PrivateSendingQueue`'s `enqueue(..)` method and storing the
    /// changed state to disk.
    public func enqueue(
        secret: PrivateSendingQueueSecret,
        message: MultiAnonymousBox<UserToCoverNodeMessageData>
    ) async throws -> HintHmac {
        guard var queue = try await loadQueue() else {
            throw PrivateSendingQueueRepositoryError.queueNotAvailable
        }
        let hint = try queue.enqueue(secret: secret, message: message)
        try await saveQueue(queue)
        return hint
    }

    /// Dequeues a message by calling the the `PrivateSendingQueue`'s `dequeue(..)` method and storing the changed state
    /// to disk.
    func dequeue(_ coverMessageFactory: CoverMessageFactory) async throws
        -> MultiAnonymousBox<UserToCoverNodeMessageData> {
        var queue = try await loadOrInitialiseQueue(coverMessageFactory)
        let message = try queue.sendHeadMessageAndPushNewCoverMessage(coverMessageFactory)

        try await saveQueue(queue)
        return message
    }

    /// Peeks a message by calling the the `PrivateSendingQueue`'s `peek(..)` method.
    public func peek() async throws -> MultiAnonymousBox<UserToCoverNodeMessageData> {
        guard var queue = try await loadQueue() else {
            throw PrivateSendingQueueRepositoryError.queueNotAvailable
        }
        let message = try queue.peek()
        return message
    }

    /// Checks a message in still in the outbound queue using `PrivateSendingQueue`'s `isMessageStillInQueue(..)`
    /// method.
    public func isMessageInQueue(hint: HintHmac) async throws -> Bool {
        guard let queue = try await loadQueue() else {
            throw PrivateSendingQueueRepositoryError.queueNotAvailable
        }
        let message = queue.isMessageStillInQueue(hint: hint)
        return message
    }

    public func loadQueue() async throws -> PrivateSendingQueue? {
        guard let bytes = try? storageManager.readFile(file: psqFile) else {
            return nil
        }
        return try PrivateSendingQueue.fromBytes(bytes: bytes)
    }

    @discardableResult
    public func saveQueue(_ currentQueue: PrivateSendingQueue?) async throws -> PrivateSendingQueue {
        guard let currentQueue else {
            throw PrivateSendingQueueRepositoryError.queueNotAvailable
        }

        let bytes = try currentQueue.serialize()
        try StorageManager.shared.writeFile(file: psqFile, data: bytes)

        try await updateHintsInFlight(currentQueue)
        return currentQueue
    }

    /// Publishes the current set of hints that are waiting to be sent to the UI process. This is then used
    /// to indicate which messages are "pending" and which are "sent".
    public func updateHintsInFlight(_ psq: PrivateSendingQueue) async throws {
        let hints = Set(psq.mHints)
        await MainActor.run {
            hintsInFlight.removeAll()
            hintsInFlight.insert(contentsOf: hints, at: 0)
        }
    }
}
