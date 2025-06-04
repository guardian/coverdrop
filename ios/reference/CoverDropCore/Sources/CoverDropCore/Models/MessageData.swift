import Combine
import Foundation

public enum Message: Codable, Equatable, Hashable, Comparable {
    case outboundMessage(message: OutboundMessageData)
    case incomingMessage(message: IncomingMessageType)

    public func getDate() -> Date {
        switch self {
        case let .incomingMessage(message: incomingMessage):
            switch incomingMessage {
            case let .handoverMessage(message: message):
                return message.timestamp
            case let .textMessage(message: message):
                return message.dateReceived
            }
        case let .outboundMessage(message: message):
            return message.dateQueued
        }
    }
}

public extension Set where Element == Message {
    /// Returns the oldest date of the messages that are in the `.soonToBeExpired` state
    func maybeExpiryDate() -> String? {
        let oldestFirst = sorted { $0.getDate() > $1.getDate() }
        for message in oldestFirst {
            let state = try? getExpiryState(messageDate: message.getDate())
            if case let .soonToBeExpired(expiryCountdownString) = state {
                if let expiryCountdownString = expiryCountdownString {
                    return expiryCountdownString
                }
            }
        }
        return nil
    }
}

enum OutboundMessageError: Error {
    case missingRecipientError
}

public class OutboundMessageData: Hashable, Codable, Comparable, ObservableObject {
    public var recipient: JournalistData
    public var messageText: String
    public var dateQueued: Date
    public var hint: HintHmac

    private enum CodingKeys: String, CodingKey {
        case recipient, messageText, dateQueued, hint
    }

    public required init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        recipient = try container.decode(JournalistData.self, forKey: .recipient)
        messageText = try container.decode(String.self, forKey: .messageText)
        dateQueued = try container.decode(Date.self, forKey: .dateQueued)
        hint = try container.decode(HintHmac.self, forKey: .hint)
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encode(recipient, forKey: .recipient)
        try container.encode(messageText, forKey: .messageText)
        try container.encode(dateQueued, forKey: .dateQueued)
        try container.encode(hint, forKey: .hint)
    }

    public static func == (lhs: OutboundMessageData, rhs: OutboundMessageData) -> Bool {
        return lhs.messageText == rhs.messageText &&
            lhs.recipient == rhs.recipient &&
            lhs.dateQueued == rhs.dateQueued
    }

    public func hash(into hasher: inout Hasher) {
        hasher.combine(dateQueued)
        hasher.combine(recipient)
        hasher.combine(messageText)
    }

    public static func < (lhs: OutboundMessageData, rhs: OutboundMessageData) -> Bool {
        return lhs.dateQueued < rhs.dateQueued
    }

    public init(
        recipient: JournalistData,
        messageText: String,
        dateQueued: Date,
        hint: HintHmac
    ) {
        self.recipient = recipient
        self.messageText = messageText
        self.dateQueued = dateQueued
        self.hint = hint
    }
}

extension OutboundMessageData: CustomStringConvertible {
    public var description: String {
        """
             recipient: \(recipient)
             messageText: \(messageText)
             dateQueued: \(dateQueued)
        """
    }
}

public enum IncomingMessageType: Hashable, Codable, Comparable {
    case textMessage(message: IncomingMessageData)
    case handoverMessage(message: HandoverMessageData)
}

public struct HandoverMessageData: Hashable, Codable, Comparable {
    public static func < (lhs: HandoverMessageData, rhs: HandoverMessageData) -> Bool {
        return lhs.handoverTo < rhs.handoverTo &&
            lhs.sender < rhs.sender &&
            lhs.timestamp < rhs.timestamp
    }

    public static func == (lhs: HandoverMessageData, rhs: HandoverMessageData) -> Bool {
        return lhs.handoverTo == rhs.handoverTo &&
            lhs.sender == rhs.sender &&
            lhs.timestamp == rhs.timestamp
    }

    init?(sender: JournalistData, timestamp: Date, handoverTo: String) {
        if handoverTo.count > Constants.maxJournalistIdentityLen {
            return nil
        }
        self.handoverTo = handoverTo
        self.sender = sender
        self.timestamp = timestamp
    }

    public var sender: JournalistData
    public var timestamp: Date
    public var handoverTo: String
}

public struct IncomingMessageData: Hashable, Codable, Comparable {
    public var sender: JournalistData
    public var messageText: String
    public var dateReceived: Date
    public var deadDropId: Int

    public static func < (lhs: IncomingMessageData, rhs: IncomingMessageData) -> Bool {
        return lhs.dateReceived < rhs.dateReceived
    }

    public init(sender: JournalistData, messageText: String, dateReceived: Date, deadDropId: Int = 0) {
        self.sender = sender
        self.messageText = messageText
        self.dateReceived = dateReceived
        self.deadDropId = deadDropId
    }
}
