import SwiftUI

public enum ExpiryState: Equatable {
    case expired
    case soonToBeExpired(expiryCountdownString: String?)
    case fresh
}

/// A message is considered to be expiring if it is either `expired` (e.g. just over the 14 day mark, but not deleted
/// yet)  or within 48 hours of being expiring, that is `soonToBeExpired`. Otherwise, it is `fresh`.
func getExpiryState(messageDate: Date) throws -> ExpiryState {
    let now = DateFunction.currentTime()
    let beforeThisIsExpired = try now.minusSeconds(Constants.messageValidForDurationInSeconds)
    let beforeThisIsSoonExpiring = try beforeThisIsExpired.plusSeconds(Constants.messageExpiryWarningInSeconds)

    if messageDate >= beforeThisIsSoonExpiring {
        return .fresh
    } else if messageDate >= beforeThisIsExpired {
        let ttl = beforeThisIsExpired.distance(to: messageDate)
        let formatter = DateComponentsFormatter()
        formatter.unitsStyle = .abbreviated
        formatter.allowedUnits = [.hour]
        return .soonToBeExpired(
            expiryCountdownString: formatter.string(from: ttl)
        )
    } else {
        return .expired
    }
}

/// A shared model for a message rendered in the UI.
public enum UiMessage {
    case incoming(messageText: String, dateReceived: Date, state: ExpiryState)
    case outgoing(messageText: String, dateQueued: Date, state: ExpiryState, isPending: Bool)

    public var messageText: String {
        switch self {
        case let .incoming(messageText, _, _): return messageText
        case let .outgoing(messageText, _, _, _): return messageText
        }
    }

    public var expiryState: ExpiryState {
        switch self {
        case let .incoming(_, _, state): return state
        case let .outgoing(_, _, state, _): return state
        }
    }

    public var date: Date {
        switch self {
        case let .incoming(_, dateReceived, _): return dateReceived
        case let .outgoing(_, dateQueued, _, _): return dateQueued
        }
    }

    public var isOutgoing: Bool {
        switch self {
        case .incoming: return false
        case .outgoing: return true
        }
    }

    public var isIncoming: Bool {
        return !isOutgoing
    }

    public var isMessagePending: Bool {
        switch self {
            case .incoming(_, _, _): return false
            case let .outgoing(_, _, _, pending): return pending
        }
    }
}

public extension Message {
    /// Turns the data-storage `Message` into the `UiMessage` which is augmented by a
    /// dynamic `isPending` flag for outgoing messages and an `ExpiryState`.
    /// Non-text messages are ignored and replaced by `nil`.
    func toUiMessage(hintsInFlight: [HintHmac]) throws -> UiMessage? {
        switch self {
        case let .incomingMessage(message):
            switch message {
            case .handoverMessage:
                return nil
            case let .textMessage(message):
                return try UiMessage.incoming(
                    messageText: message.messageText,
                    dateReceived: message.dateReceived,
                    state: getExpiryState(messageDate: message.dateReceived)
                )
            }
        case let .outboundMessage(message):
            return try UiMessage.outgoing(
                messageText: message.messageText,
                dateQueued: message.dateQueued,
                state: getExpiryState(messageDate: message.dateQueued),
                isPending: hintsInFlight.contains(where: { $0 == message.hint })
            )
        }
    }
}
