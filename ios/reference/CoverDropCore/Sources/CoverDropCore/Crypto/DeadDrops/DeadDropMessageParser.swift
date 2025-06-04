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
        } else if firstByte == Constants.flagJ2UMessageTypeHandover {
            return parseHandoverMessage(
                messageBytes: remainingMessageBytes,
                journalistData: journalistData,
                deadDropId: deadDropId,
                dateReceived: dateReceived
            )
        } else {
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

    private static func parseHandoverMessage(
        messageBytes: [UInt8],
        journalistData: JournalistData,
        deadDropId _: Int,
        dateReceived: Date
    ) -> Message? {
        guard let endPositionOfJournalistIdentity = messageBytes.firstIndex(of: 0x00),
              Constants.maxJournalistIdentityLen >= endPositionOfJournalistIdentity,
              let journalistIdentityString = String(
                  bytes: Array(messageBytes[1 ..< endPositionOfJournalistIdentity]),
                  encoding: .utf8
              ),
              journalistIdentityString.count <= Constants.maxJournalistIdentityLen,
              let handoverMessage = HandoverMessageData(
                  sender: journalistData,
                  timestamp: dateReceived,
                  handoverTo: journalistIdentityString
              ) else { return nil }
        return .incomingMessage(message:
            .handoverMessage(message: handoverMessage))
    }
}
