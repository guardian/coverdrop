import Combine
import Foundation

public enum SecretData {
    case lockedSecretData(lockedData: LockedSecretData)
    case unlockedSecretData(unlockedData: UnlockedSecretData)
}

public class LockedSecretData: Codable {}

public class UnlockedSecretData: Codable, Equatable, ObservableObject {
    public var uuid: UUID = .init()
    public var messageMailbox: Set<Message>
    public var userKey: EncryptionKeypair<User>
    public var privateSendingQueueSecret: PrivateSendingQueueSecret

    @Published public var publishedMessageMailbox: Set<Message>

    enum CodingKeys: CodingKey {
        case uuid, messageMailbox, userKey, privateSendingQueueSecret
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)

        try container.encode(uuid, forKey: .uuid)
        try container.encode(messageMailbox, forKey: .messageMailbox)
        try container.encode(userKey, forKey: .userKey)
        try container.encode(privateSendingQueueSecret, forKey: .privateSendingQueueSecret)
    }

    public required init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        uuid = try container.decode(UUID.self, forKey: .uuid)
        messageMailbox = try container.decode(Set<Message>.self, forKey: .messageMailbox)
        userKey = try container.decode(EncryptionKeypair<User>.self, forKey: .userKey)
        privateSendingQueueSecret = try container.decode(
            PrivateSendingQueueSecret.self,
            forKey: .privateSendingQueueSecret
        )

        publishedMessageMailbox = messageMailbox
    }

    public init(
        uuid: UUID = UUID(),
        messageMailbox: Set<Message>,
        userKey: EncryptionKeypair<User>,
        privateSendingQueueSecret: PrivateSendingQueueSecret
    ) {
        self.uuid = uuid
        self.messageMailbox = messageMailbox
        self.userKey = userKey
        self.privateSendingQueueSecret = privateSendingQueueSecret

        publishedMessageMailbox = messageMailbox
    }

    static func createEmpty() throws -> UnlockedSecretData {
        return try UnlockedSecretData(
            messageMailbox: Set<Message>(),
            userKey: EncryptionKeypair<User>.generateEncryptionKeypair(),
            privateSendingQueueSecret: PrivateSendingQueueSecret.fromSecureRandom()
        )
    }

    public func addMessage(message: Message) async {
        messageMailbox.insert(message)
        _ = await MainActor.run {
            publishedMessageMailbox.insert(message)
        }
    }

    public func addMessages(messages: Set<Message>) async {
        messageMailbox.formUnion(messages)
        _ = await MainActor.run {
            publishedMessageMailbox.formUnion(messages)
        }
    }

    public func removeExpiredMessages(cutoff: Date) async {
        let expiredMessages = messageMailbox.filter { $0.getDate() < cutoff }
        messageMailbox.subtract(expiredMessages)
        _ = await MainActor.run {
            publishedMessageMailbox.subtract(expiredMessages)
        }
    }

    public static func == (lhs: UnlockedSecretData, rhs: UnlockedSecretData) -> Bool {
        return lhs.messageMailbox == rhs.messageMailbox &&
            lhs.userKey.publicKey.key == rhs.userKey.publicKey.key &&
            lhs.userKey.secretKey.key == rhs.userKey.secretKey.key &&
            lhs.privateSendingQueueSecret == rhs.privateSendingQueueSecret
    }
}
