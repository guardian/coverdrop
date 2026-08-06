import Foundation

enum DeadDropMessageParser {
    static func parseMessage(
        messageBytes: [UInt8],
        journalistData: JournalistData,
        deadDropId: Int,
        dateReceived: Date
    ) -> Message? {
        guard let firstByte = messageBytes.first else { return nil }
        let remainingMessageBytes = Array(messageBytes.suffix(Constants.messagePaddingLen))
        if firstByte == Constants.flagJ2UMessageTypeMessage {
            return parseTextMessage(
                messageBytes: remainingMessageBytes,
                journalistData: journalistData,
                deadDropId: deadDropId,
                dateReceived: dateReceived
            )
        } else {
            // this includes the deprecated handover flag (0x01) which must not
            // trigger any logic
            return nil
        }
    }

    private static func parseTextMessage(
        messageBytes: [UInt8],
        journalistData: JournalistData,
        deadDropId: Int,
        dateReceived: Date
    ) -> Message? {
        if messageBytes.count != Constants.messagePaddingLen {
            Debug.println("message bytes did not match messagePaddingLen")
            return nil
        }
        guard let extractedMessage = try? PaddedCompressedString(value: Array(messageBytes)).toString() else {
            return nil
        }
        return .incomingMessage(message: .textMessage(message: IncomingMessageData(
            sender: journalistData,
            messageText: extractedMessage,
            dateReceived: dateReceived,
            deadDropId: deadDropId
        )))
    }
}
