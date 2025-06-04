import CoverDropCore
import SwiftUI

public struct ActiveConversation: Equatable {
    let recipient: JournalistData
    let lastMessageUpdated: Date

    var messages: Set<Message> = []

    var formattedLastMessageUpdated: String {
        lastMessageUpdated.formatted(date: .abbreviated, time: .shortened)
    }
}

public struct InactiveConversation: Equatable {
    let recipient: JournalistData
    var messages: Set<Message> = []
}

@MainActor
class InboxViewModel: ObservableObject {
    private var mailbox: Set<Message>? {
        guard case let .unlockedSecretData(unlockedData: data) = lib.secretDataRepository.getSecretData() else {
            return nil
        }
        return data.messageMailbox
    }

    var activeConversation: ActiveConversation? {
        InboxViewModel.findActiveConversation(in: mailbox)
    }

    var inactiveConversations: [InactiveConversation]? {
        InboxViewModel.findInactiveMessages(in: mailbox)
    }

    @ObservedObject var lib: CoverDropLibrary

    init(lib: CoverDropLibrary) {
        self.lib = lib
    }

    /// Finds the most recent conversation in a mailbox, and returns an Active Conversation. If there are no messages in
    /// the mailbox, this returns nil.
    static func findActiveConversation(in mailbox: Set<Message>?) -> ActiveConversation? {
        guard let existingMailbox = mailbox else {
            return nil
        }
        // 1. If we only have outbound messages there should just be a single recipient
        // so return the active conversation as the recipient of this message
        let onlyOutbound = existingMailbox.allSatisfy {
            if case .outboundMessage = $0 { return true }
            return false
        }

        if onlyOutbound {
            let messages = existingMailbox.sorted(by: >)
            if let message = messages.first {
                if case let .outboundMessage(data) = message {
                    return ActiveConversation(
                        recipient: data.recipient,
                        lastMessageUpdated: data.dateQueued,
                        messages: Set(messages)
                    )
                }
            }
        }

        // 1. Remove messages sent from user
        let mailboxRemovingOutbound = mailbox?.filter {
            guard case .incomingMessage = $0 else { return false }
            return true
        }

        // 2. Find most recent message from remaining message (this will identify the active conversation)
        guard let mostRecentMessage = mailboxRemovingOutbound?.sorted(by: >).first else { return nil }

        if case let .incomingMessage(message: incomingMessageType) = mostRecentMessage {
            if case let .textMessage(message: incomingMessage) = incomingMessageType {
                let allMessagesInConversation = messagesForRecipient(
                    recipient: incomingMessage.sender,
                    mailbox: mailbox
                )
                return ActiveConversation(
                    recipient: incomingMessage.sender,
                    lastMessageUpdated: incomingMessage.dateReceived,
                    messages: allMessagesInConversation
                )
            } else {
                return nil
            }
        }
        return nil
    }

    /// Finds the inactive conversations in the inbox. If there are no messages in the mailbox, this returns nil. If
    /// there is 1 message in the mailbox, this returns nil - since 1 message would be considered the active
    /// conversation.
    static func findInactiveMessages(in mailbox: Set<Message>?) -> [InactiveConversation]? {
        // 1. Remove all outbound messages
        let mailboxRemovingOutbound = mailbox?.filter {
            guard case .incomingMessage = $0 else { return false }
            return true
        }

        // 2. Remove messages from the active message recipient
        guard case let .incomingMessage(incomingMessage) = mailboxRemovingOutbound?.sorted(by: >).first else {
            return nil
        }
        guard case let .textMessage(activeMessage) = incomingMessage else {
            return nil
        }

        let inactiveMailbox = mailboxRemovingOutbound?.filter {
            if case let .incomingMessage(message: incomingMessageType) = $0,
               case let .textMessage(message) = incomingMessageType,
               message.sender == activeMessage.sender {
                return false
            }
            return true
        }
        guard let inactiveMailbox, inactiveMailbox.count > 0 else { return nil }

        // 3. Find the remaining recipients
        let recipients: [JournalistData] = inactiveMailbox.compactMap {
            guard case let .incomingMessage(message: incomingMessageType) = $0 else { return nil }
            guard case let .textMessage(activeMessage) = incomingMessageType else {
                return nil
            }
            return activeMessage.sender
        }

        // 4. Return an array of inactive threads, in no particular order
        return recipients.map {
            let allMessagesInConversation = messagesForRecipient(recipient: $0, mailbox: mailbox)
            return InactiveConversation(recipient: $0, messages: allMessagesInConversation)
        }
    }

    static func messagesForRecipient(recipient: JournalistData, mailbox: Set<Message>?) -> Set<Message> {
        return mailbox?.filter {
            switch $0 {
            case let .outboundMessage(message: outbound):
                return outbound.recipient == recipient

            case let .incomingMessage(message: incoming):
                if case let .textMessage(message: incomingMessageData) = incoming {
                    return incomingMessageData.sender == recipient
                } else {
                    return false
                }
            }
        } ?? Set()
    }

    public func deleteAllMessagesAndCurrentSession(conversationViewModel: ConversationViewModel) async throws {
        try await lib.secretDataRepository.deleteVault()
        await conversationViewModel.clearModelDataAndLock()
    }
}
