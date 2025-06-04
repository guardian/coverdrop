import CoverDropCore
import Gzip
import SwiftUI

public enum MessageComposeError: Error {
    case textTooLong, invalidCharacter, compressionFailed, unknownError
}

@MainActor class ConversationViewModel: ObservableObject {
    @ObservedObject var lib: CoverDropLibrary
    @ObservedObject var privateSendingQueue = PrivateSendingQueueRepository.shared

    @Published var messageRecipient: JournalistData?
    @Published private(set) var recipients: MessageRecipients?

    @Published var message = ""
    @Published var state = State.initial

    public init(
        lib: CoverDropLibrary
    ) {
        // We will always set messageRecipient to the supplied one
        // but set it with the defaul journalist if we can get any message reciepients from the keys
        // and the supplied recipient is nil
        self.lib = lib
        state = .loading

        messageRecipient = nil

        // It is a valid scenario that there are no recipients available from the public keys data (ie they have all
        // expired)
        // but we still was the user to be able to view a conversation
        if messageRecipient == nil {
            if let verifiedPublicKeys = try? lib.publicDataRepository.getVerifiedKeys(),
               let messageRecipientsFromKeys = try? MessageRecipients(
                   verifiedPublicKeys: verifiedPublicKeys,
                   excludingDefaultRecipient: false
               ) {
                recipients = messageRecipientsFromKeys
                if messageRecipientsFromKeys.defaultRecipient != nil {
                    messageRecipient = messageRecipientsFromKeys.defaultRecipient
                }
            } else {
                messageRecipient = recipients?.journalists.first
            }
        }
        state = .ready
    }

    enum State: Equatable {
        case initial, loading, ready, sending
        case error(message: String)
    }

    /// This returns the current conversation as a list of `Message` based on the message recipient
    var currentConversation: [Message] {
        switch lib.secretDataRepository.getSecretData() {
        case let .unlockedSecretData(unlockedData: unlockedData):
            let filteredMailbox: [Message] = unlockedData.messageMailbox.filter { message in
                switch message {
                case let .incomingMessage(message: messageData):
                    if case let .textMessage(message: incomingMessage) = messageData {
                        return incomingMessage.sender == messageRecipient
                    } else {
                        return false
                    }
                case let .outboundMessage(message: messageData):
                    return messageData.recipient == messageRecipient
                }
            }
            return filteredMailbox.sorted(by: { message1, message2 -> Bool in
                message1.getDate() < message2.getDate()
            })
        case _:
            return []
        }
    }

    /// This returns the current conversation as a list of `UiMessage` based on the message
    /// recipient (as per `currentCoversation`) and sets pending flags according to the
    /// private sending queue.
    var currentConversationForUi: [UiMessage?] {
        return currentConversation.map {
            try? $0.toUiMessage(hintsInFlight: privateSendingQueue.hintsInFlight)
        }
    }

    /// This checks to see if the current conversation in active or inactive
    /// We currenly only allow the user to reply to the active conversation, which restricts
    /// the user to having one conversation at the time.
    /// We check to see if the most recent message from the current conversation is in the active conversation
    func isCurrentConversationActive(maybeActiveConversation: ActiveConversation?) -> Bool {
        if let activeConversation = maybeActiveConversation,
           let lastConversationMessage = currentConversation.last {
            return activeConversation.messages.contains(where: { $0 == lastConversationMessage })
        } else {
            return false
        }
    }

    // This sends a message to the selected message recipient using the covernode public key
    // This handles any errors and updates the view state.
    @MainActor
    func sendMessage() async throws {
        guard let messageRecipient else {
            state = .error(message: "No recipient selected")
            return
        }

        let dateSent = DateFunction.currentTime()

        do {
            try await lib.secretDataRepository.sendMessage(
                message,
                to: messageRecipient,
                dateSent: dateSent
            )
        } catch {
            state = .error(message: error.localizedDescription)
        }
    }

    // We want to disable the send button when the message box is empty,
    // too long, no recipient is selected, or we are in the sending state
    var sendButtonDisabled: Bool {
        switch state {
        case .sending:
            return false
        case _:
            return message.isEmpty || messageIsTooLong || messageRecipient == nil
        }
    }

    var messageIsTooLong: Bool {
        do {
            try PaddedCompressedString.compressCheckingLength(from: message)
            return false
        } catch PaddedCompressedStringError.compressedStringTooLong {
            return true
        } catch {
            return false
        }
    }

    /// Get the compressed size of the current message, if its greater than zero
    /// return this as a percentage of the total padded message length
    /// otherwise return 0
    var messageLengthProgressPercentage: Result<Double, MessageComposeError> {
        let padToSize = Constants.messagePaddingLen
        do {
            let (_, compressedSize) = try PaddedCompressedString.compressCheckingLength(from: message)
            // Check the size is greater than 0 to avoid returning `Infinity`
            if compressedSize > 0 {
                if message.count < 30 {
                    // So our compression algo requires 20 bytes overhead when compressing 1 byte,
                    // So a single character outputs 4% of progress complete.
                    // To smooth this out the percentage progress bar
                    // any message byte length below 30 has the percentage calculated by dividing the value by 4
                    // then we round up the first 4 characters so they are consider 1%
                    return .success(ceil(Double(message.count) / 4))
                }
                let percentage = Double(compressedSize) / Double(padToSize) * 100
                return .success(percentage)
            } else {
                return .success(0)
            }
        } catch PaddedCompressedStringError.compressedStringTooLong {
            return .failure(MessageComposeError.textTooLong)
        } catch PaddedCompressedStringError.invaidUTF8StringConversion {
            return .failure(MessageComposeError.invalidCharacter)
        } catch is GzipError {
            return .failure(MessageComposeError.compressionFailed)
        } catch {
            return .failure(MessageComposeError.unknownError)
        }
    }

    // This clears the value of `message`
    // Used to clear the TextEditor input box when a message is sent
    // Or when the users first focuses on the input
    func clearMessage() {
        message = ""
        state = .ready
    }

    // This clears the value of `message`, removes the current recipient and locks the secret data.
    // These are coupled to avoid developer error in doing them seperatly.
    // This is called in the various places the user can logout or delete messages.
    public func clearModelDataAndLock() async {
        messageRecipient = nil
        clearMessage()
        try? await lib.secretDataRepository.lock()
    }

    static let messageDateFormat: DateFormatter = {
        let formatter = DateFormatter()
        formatter.dateFormat = "dd MMMM yyyy HH:mm z"
        return formatter
    }()

    func isMostRecentMessageFromUser() -> Bool {
        switch lib.secretDataRepository.getSecretData() {
        case let .unlockedSecretData(unlockedData: unlockedData):
            if let recentMessage: Message = unlockedData.messageMailbox.sorted(by: >).first {
                switch recentMessage {
                case .outboundMessage:
                    return true
                case _:
                    return false
                }
            } else {
                return false
            }
        case _:
            return false
        }
    }

    func isMostRecentMessagePending() -> Bool {
        guard let recentMessage = currentConversationForUi.last else { return false }
        return recentMessage?.isMessagePending ?? false
    }

}
