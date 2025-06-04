import CryptoKit
import Foundation
import Sodium

// These are all `Int32` just to make behaviour consistant across devices
// as `Int` bit length will vary between platforms (64 bit or 32 bit depending on the device)
// as we are serialising this to binary we want the length to be fixed.
// Also having them all the same type makes arithmatic more straight forward in the code.

let privateSendingQueueSecretLenBytes: Int32 = 16
let hintSizeBytes: Int32 = 32

let currentMessagesIntBytes: Int32 = 4
let messageSizeIntBytes: Int32 = 4

enum PrivateSendingQueueError: Error {
    case serializationSizeDidNotMatch
    case queueIsFull
    case queueIsEmpty
    case deserializationBufferSizeIncorrect
    case queueSizesVary
    case messageOrHintSizeIncorrect
    case unexpectedEndOfStream
}

// This is a Hmac of the encrypted message ciphertext
// and is used internally within the application to idetify if a message
// is still in the queue, without needed to decrypt the enqueued message to check the value.
public struct HintHmac: Codable, Equatable, Hashable {
    public let hint: [UInt8]

    public init(hint: [UInt8]) {
        self.hint = hint
    }
}

///  A `PrivateSendingQueue` is a data structure to store a mix of real and cover messages. An
///  adversary cannot tell from a single snapshot how many real and how many cover messages are
///  included. However, a caller that uses a consistent secret will be able to tell how many real
///  messages are currently stored. Also, it ensures that real messages that are enqueued are placed
///  before all cover messages.
public struct PrivateSendingQueue: Equatable {
    public static func == (lhs: PrivateSendingQueue, rhs: PrivateSendingQueue) -> Bool {
        return lhs.mStorage == rhs.mStorage &&
            lhs.mHints == rhs.mHints
    }

    var totalQueueSize: Int32
    var messageSize: Int32
    private var initialMessagesAndHints: ([MultiAnonymousBox<UserToCoverNodeMessageData>], [UInt8])

    var mStorage: [MultiAnonymousBox<UserToCoverNodeMessageData>] = []
    var mHints: [HintHmac] = []

    /// Initialise a `PrivateSendingQueue`
    /// - Parameters:
    ///   - totalQueueSize: The total number of messages the queue can hold, this is an `Int32` to make cross platform
    /// serialization standard
    ///   - messageSize: The size in bytes of a single message, this is an `Int32` to make cross platform
    /// serialization standard
    ///   - initialMessagesAndHints: the inital message and hints you want to place in the queue.
    init?(
        totalQueueSize: Int32,
        messageSize: Int32,
        initialMessagesAndHints: ([MultiAnonymousBox<UserToCoverNodeMessageData>], [UInt8])
    ) {
        self.totalQueueSize = totalQueueSize
        self.messageSize = messageSize
        self.initialMessagesAndHints = initialMessagesAndHints
        mStorage = initialMessagesAndHints.0
        mHints = initialMessagesAndHints.1.chunked(into: Int(hintSizeBytes)).map { HintHmac(hint: $0) }
        if !assertInvariants() {
            return nil
        }
    }

    /// Initialise a `PrivateSendingQueue` with random data in both the message and hints blocks
    /// corresponding to the number and size of messages requested.
    /// - Parameters:
    ///   - totalQueueSize: The total number of messages the queue can hold, this is an `Int32` to make cross platform
    /// serialization standard
    ///   - messageSize: The size in bytes of a single message, this is an `Int32` to make cross platform serialization
    /// standard
    init(totalQueueSize: Int32, messageSize: Int32, coverMessageFactory: CoverMessageFactory) throws {
        self.totalQueueSize = totalQueueSize
        self.messageSize = messageSize
        initialMessagesAndHints = ([], [])
        // fill-up queue with cover messages
        while mStorage.count < totalQueueSize {
            let coverMessage = try coverMessageFactory()
            try addCoverMessageAndHint(coverMessage: coverMessage)
        }
        if !assertInvariants() {
            throw PrivateSendingQueueError.queueSizesVary
        }
    }

    /// Removes and returns the front-most message of the queue. If there were any real messages in the buffer,
    /// they would be at the front and returned before any cover messages. Afterwards the buffer
    /// is filled up to `self.size` again.
    @discardableResult
    mutating func sendHeadMessageAndPushNewCoverMessage(_ coverMessageFactory: CoverMessageFactory) throws
        -> MultiAnonymousBox<UserToCoverNodeMessageData> {
        let message = mStorage.removeFirst()
        _ = mHints.removeFirst()

        // and fill-up both
        let coverMessage = try coverMessageFactory()
        try addCoverMessageAndHint(coverMessage: coverMessage)

        if !assertInvariants() {
            throw PrivateSendingQueueError.queueSizesVary
        }
        return message
    }

    /// Returns the front-most message of the queue, but does not remove it.
    /// This is to allow us to send a message via the api, without altering the queue,
    @discardableResult
    mutating func peek() throws -> MultiAnonymousBox<UserToCoverNodeMessageData> {
        if let message = mStorage.first {
            return message
        } else {
            throw PrivateSendingQueueError.queueIsEmpty
        }
    }

    /// Returns the current number of real messages. This requires that the same `secret` is used
    /// for both `getFillLevel` and `enqueue`.
    public func getFillLevel(secret: PrivateSendingQueueSecret) -> Int {
        var fillLevel = 0
        for messageAndMessage in zip(mStorage, mHints) {
            let message = messageAndMessage.0
            let hint = messageAndMessage.1

            let hmac = PrivateSendingQueueHmac.hmac(secretKey: secret.bytes, message: message.asBytes())
            if hmac != hint.hint { break }
            fillLevel += 1
        }
        return fillLevel
    }

    public func isMessageStillInQueue(hint: HintHmac) -> Bool {
        return mHints.contains(where: { $0.hint == hint.hint })
    }

    /// Enqueues a new message. If the same `secret` is used for all calls to `enqueue`, it
    /// guarantees that: (a) the real messages are returned FIFO and (b) they are returned before
    /// any cover messages.
    ///
    /// However, if different `secret` values are used, existing real messages are not detected and
    /// will be overwritten.
    /// - Parameters:
    ///   - secret: a secret generated and stored
    ///   - message: the ecrypted message to be enqueued
    ///   - sendingQueueMessageSize: the expected message size of the private sending queue. Defaults to the value
    /// specified in the `PrivateSendignQueueConfiguration.default.messageSize`.
    /// - throws: if the queue is full  of real messages or an incorrect size
    mutating func enqueue(
        secret: PrivateSendingQueueSecret,
        message: MultiAnonymousBox<UserToCoverNodeMessageData>,
        sendingQueueMessageSize: Int32 = PrivateSendingQueueConfiguration.default.messageSize
    ) throws -> HintHmac {
        let fillLevel = getFillLevel(secret: secret)

        if fillLevel == totalQueueSize {
            throw PrivateSendingQueueError.queueIsFull
        }

        if message.asBytes().count != sendingQueueMessageSize {
            throw PrivateSendingQueueError.messageOrHintSizeIncorrect
        }

        mStorage.insert(message, at: fillLevel)
        _ = mStorage.popLast()

        let hint = HintHmac(hint: PrivateSendingQueueHmac.hmac(
            secretKey: secret.bytes,
            message: message.asBytes()
        ))

        mHints.insert(hint, at: fillLevel)
        _ = mHints.popLast()

        if !assertInvariants() {
            throw PrivateSendingQueueError.queueSizesVary
        }
        return hint
    }

    /// Takes a byte array from `byteLength` from a `Data` buffer and returns it as an `Int32`
    /// then removes the retrived bytes from the buffer.
    /// - Parameters:
    ///   - byteLength: the number of bytes to pop from the buffer
    ///   - buffer: a reference to a `Data` buffer
    /// - Returns: the value retrived from the buffer as an `Int32` note that no validation of the output is done
    static func popInt(byteLength: Int32, buffer: inout Data) throws -> Int32 {
        if buffer.count >= byteLength {
            let value = withUnsafeBytes(of: buffer[0 ..< byteLength]) { $0.load(as: Int32.self) }
            buffer.removeSubrange(0 ..< Int(byteLength))
            return value
        } else {
            throw PrivateSendingQueueError.unexpectedEndOfStream
        }
    }

    /// takes a byte array from `byteLength` from a `Data` buffer and returns it as an `Int32`
    /// then removes the retrived bytes from the buffer.
    /// - Parameters:
    ///   - byteLength: the number of bytes to pop from the buffer
    ///   - buffer: a reference to a `Data` buffer
    /// - Returns: the value retrived from the buffer as a `[UInt8]`
    static func popArray(byteLength: Int32, buffer: inout Data) throws -> [UInt8] {
        if buffer.count >= byteLength {
            let value = Array(buffer[0 ..< byteLength])
            buffer.removeSubrange(0 ..< Int(byteLength))
            return value
        } else {
            throw PrivateSendingQueueError.unexpectedEndOfStream
        }
    }

    /// Deserializes a `PrivateSendingQueue` from a `[UInt8]` that was previously
    /// created with `serialize`.
    /// - Parameter bytes: a `[UInt8]` that was previously created with `serialize`.
    ///  - Parameter sendingQueueMessageSize: the expected message size of the private sending queue. Defaults to the
    /// value specified in the `PrivateSendignQueueConfiguration.default.messageSize`.
    /// - Returns: a `PrivateSendingQueue`
    /// - throws: if the `bytes` were not able to be deserialized to the expected length
    static func fromBytes(bytes: [UInt8],
                          sendingQueueMessageSize: Int32 = PrivateSendingQueueConfiguration.default
                              .messageSize) throws -> PrivateSendingQueue {
        var buffer = Data(bytes)

        let numberOfMessages: Int32 = try popInt(byteLength: currentMessagesIntBytes, buffer: &buffer)
        let messageSizeInt: Int32 = try popInt(byteLength: messageSizeIntBytes, buffer: &buffer)
        let storageBlock: [UInt8] = try popArray(byteLength: numberOfMessages * messageSizeInt, buffer: &buffer)
        let hintsBlock: [UInt8] = try popArray(byteLength: numberOfMessages * hintSizeBytes, buffer: &buffer)

        if buffer.count != 0 { throw PrivateSendingQueueError.deserializationBufferSizeIncorrect }

        let messages: [MultiAnonymousBox<UserToCoverNodeMessageData>] = storageBlock
            .chunked(into: Int(sendingQueueMessageSize)).map { message in
                MultiAnonymousBox<UserToCoverNodeMessageData>.fromVecUnchecked(bytes: message)
            }
        let initialMessagesAndHints: ([MultiAnonymousBox<UserToCoverNodeMessageData>], [UInt8]) = (messages, hintsBlock)

        return PrivateSendingQueue(
            totalQueueSize: numberOfMessages,
            messageSize: messageSizeInt,
            initialMessagesAndHints: initialMessagesAndHints
        )!
    }

    /// Serializes all internal state into a `[UInt8]`] that can later be used with `fromBytes`.
    /// It has the following byte structure.
    /// ```
    ///   +------------------------+---------------+--------------------+---------------+
    ///   |  number Of Messages(n) | message Size  |  n Messages        | n Hints       |
    ///   +------------------------+---------------+--------------------+---------------+
    ///           ^                    ^                  ^                    ^
    ///           |                    |                  |                    |
    ///          4B                   4B             n * message Size B   n * hint Size
    /// ```
    /// - Returns: a `[UInt8]` of the internal state
    /// - throws: if the serialization did not match the expected size
    ///   or the message and hint queues were not able to be serialized into the `Int32` allocated space.
    func serialize() throws -> [UInt8] {
        let totalMessageSize = totalQueueSize * messageSize
        let totalHintSize = totalQueueSize * hintSizeBytes
        let expectedSize = currentMessagesIntBytes + messageSizeIntBytes + totalMessageSize + totalHintSize

        var buffer = Data(capacity: Int(expectedSize))

        let numberBytes: [UInt8] = Array(withUnsafeBytes(of: totalQueueSize) { Data($0) })
        let messageSizeByteArray: [UInt8] = Array(withUnsafeBytes(of: messageSize) { Data($0) })

        if numberBytes.count != currentMessagesIntBytes || messageSizeByteArray.count != messageSizeIntBytes {
            throw PrivateSendingQueueError.messageOrHintSizeIncorrect
        }

        buffer.append(contentsOf: numberBytes)
        buffer.append(contentsOf: messageSizeByteArray)

        for message in mStorage {
            buffer.append(contentsOf: message.asBytes())
        }

        for hint in mHints {
            buffer.append(contentsOf: hint.hint)
        }

        let byteArray = Array(buffer)

        if byteArray.count != expectedSize {
            throw PrivateSendingQueueError.serializationSizeDidNotMatch
        }

        return byteArray
    }

    /// Adds random message and hints
    private mutating func addCoverMessageAndHint(coverMessage: MultiAnonymousBox<UserToCoverNodeMessageData>) throws {
        try addMessageAndHint(
            message: coverMessage,
            hint: HintHmac(hint: Sodium().randomBytes.buf(length: Int(hintSizeBytes))!)
        )
        // }
    }

    /// Adds message and hint to the internal storage
    /// - Parameters:
    ///   - message: the message to be added
    ///   - hint: the hint hash of the message and secret
    private mutating func addMessageAndHint(
        message: MultiAnonymousBox<UserToCoverNodeMessageData>,
        hint: HintHmac
    ) throws {
        if message.asBytes().count != messageSize || hint.hint.count != hintSizeBytes {
            throw PrivateSendingQueueError.messageOrHintSizeIncorrect
        }

        mStorage.append(message)
        mHints.append(hint)
    }

    /// Checks that all storage elements match in size and counts
    private func assertInvariants() -> Bool {
        return mStorage.count == totalQueueSize &&
            mHints.count == totalQueueSize &&
            mStorage.allSatisfy { $0.asBytes().count == messageSize } &&
            mHints.allSatisfy { $0.hint.count == hintSizeBytes }
    }
}

public enum PrivateSendingQueueHmac {
    /// Generates a SHA256 HMAC of the provided message with the supplied secret
    /// - Parameters:
    ///   - secretKey: the secret for hashing messages
    ///   - message: the message to hash
    /// - Returns: the computed HMAC
    public static func hmac(secretKey: [UInt8], message: [UInt8]) -> [UInt8] {
        let key = SymmetricKey(data: Data(secretKey))
        let hashInt = HMAC<SHA256>.authenticationCode(for: Data(message), using: key)
        let byteArray = Array(withUnsafeBytes(of: hashInt) { Data($0) })
        return byteArray
    }
}

enum PrivateSendingQueueSecretError: Error {
    case unableToCreateKeyFromSecureRandom
    case bytesLengthIncorrect
}

/// A secret used to hash real messages on the `PrivateSendingQueue`
public struct PrivateSendingQueueSecret: Codable {
    public var bytes: [UInt8]

    init?(bytes: [UInt8]) {
        if bytes.count != privateSendingQueueSecretLenBytes {
            return nil
        }
        self.bytes = bytes
    }

    public static func fromSecureRandom() throws -> PrivateSendingQueueSecret {
        guard let bytes = Sodium().randomBytes.buf(length: Int(privateSendingQueueSecretLenBytes)) else {
            throw PrivateSendingQueueSecretError.unableToCreateKeyFromSecureRandom
        }
        guard let secret = PrivateSendingQueueSecret(bytes: bytes) else {
            throw PrivateSendingQueueSecretError.bytesLengthIncorrect
        }
        return secret
    }

    func deserialize(bytes _: [UInt8]) -> PrivateSendingQueueSecret? {
        return PrivateSendingQueueSecret(bytes: bytes)
    }

    func serialize() -> [UInt8] {
        return bytes
    }

    static func == (lhs: PrivateSendingQueueSecret, rhs: PrivateSendingQueueSecret) -> Bool {
        return lhs.bytes == rhs.bytes
    }

    func hashCode() -> Int {
        return bytes.hashValue
    }
}
