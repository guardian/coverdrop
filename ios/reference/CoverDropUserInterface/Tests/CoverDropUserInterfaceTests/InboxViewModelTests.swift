@testable import CoverDropCore
@testable import CoverDropUserInterface
import Sodium
import XCTest

// swiftlint:disable force_try
@MainActor
final class InboxViewModelTests: XCTestCase {
    // the hour is the number of hours to add / subtract from noon
    private func testDate(hourFromNoon hour: Int = 0) -> Date {
        if let date = try? XCTUnwrap(DateComponents(
            calendar: .current,
            year: 2023,
            month: 05,
            day: 01,
            hour: 12 + hour
        ).date) { return date } else { return DateFunction.currentTime() }
    }

    private static let recipients = try! MessageRecipients(
        verifiedPublicKeys: PublicKeysHelper.shared.testKeys,
        excludingDefaultRecipient: false
    )

    private let firstTestJournalist = recipients.journalists[0]
    private let secondTestJournalist = recipients.journalists[1]
    private let thirdTestJournalist = recipients.desks[0]

    func testActiveConversatoinWithNoMessages() {
        // GIVEN a mailbox with no messages
        let mailbox: Set<Message> = []

        // WHEN when finding the active conversation
        let activeMessage = InboxViewModel.findActiveConversation(in: mailbox)

        // THEN no active conversations are found
        XCTAssertNil(activeMessage)
    }

    func testActiveConversatoinWithSingleOutboundMessages() throws {
        let messageText = "hello!"

        let privateSendingQueueSecret = try PrivateSendingQueueSecret.fromSecureRandom()
        // GIVEN a mailbox with no messages
        let recentOutboundMessage = Message.outboundMessage(
            message: OutboundMessageData(
                recipient: firstTestJournalist,
                messageText: messageText,
                dateQueued: testDate(),
                hint: HintHmac(hint: PrivateSendingQueueHmac.hmac(
                    secretKey: privateSendingQueueSecret.bytes,
                    message: messageText.asBytes()
                ))
            )
        )
        // WHEN when finding the active conversation
        let activeMessage = InboxViewModel.findActiveConversation(in: [recentOutboundMessage])

        // THEN no active conversations are found
        XCTAssertNotNil(activeMessage)
    }

    // Note this case should not be allowed within the UI
    // You should not be able to send outbound messages to multiple journalists without receiving a reply
    func testActiveConversatoinWithTwoOutboundMessagesOnly() throws {
        let privateSendingQueueSecret = try PrivateSendingQueueSecret.fromSecureRandom()

        let messageText = "hello!"
        // GIVEN a mailbox with no messages
        let recentOutboundMessage = Message.outboundMessage(
            message: OutboundMessageData(
                recipient: firstTestJournalist,
                messageText: messageText,
                dateQueued: testDate(hourFromNoon: 0),
                hint: HintHmac(hint: PrivateSendingQueueHmac.hmac(
                    secretKey: privateSendingQueueSecret.bytes,
                    message: messageText.asBytes()
                ))
            )
        )
        // This message is send earlier than the first one
        let recentOutboundMessage2 = Message.outboundMessage(
            message: OutboundMessageData(
                recipient: secondTestJournalist,
                messageText: messageText,
                dateQueued: testDate(hourFromNoon: -1),
                hint: HintHmac(hint: PrivateSendingQueueHmac.hmac(
                    secretKey: privateSendingQueueSecret.bytes,
                    message: messageText.asBytes()
                ))
            )
        )
        // WHEN when finding the active conversation
        let activeMessage = InboxViewModel.findActiveConversation(in: [recentOutboundMessage2, recentOutboundMessage])

        // THEN no active conversations are found
        XCTAssertEqual(activeMessage?.recipient, firstTestJournalist)
    }

    func testActiveConversationWith1Messages() {
        let messageText = "hello!"
        // GIVEN a mailbox with a single message
        let recentMessage = Message.incomingMessage(
            message: .textMessage(
                message: IncomingMessageData(
                    sender: firstTestJournalist,
                    messageText: messageText,
                    dateReceived: testDate()
                )
            )
        )

        // WHEN finding the active conversation
        let activeMessage = InboxViewModel.findActiveConversation(in: [recentMessage])

        let expectedActiveMessage = ActiveConversation(
            recipient: firstTestJournalist,
            lastMessageUpdated: testDate(),
            messages: [recentMessage]
        )

        // THEN the active conversation is the only message in the mailbox
        XCTAssertEqual(activeMessage, expectedActiveMessage)
    }

    func testActiveConversationWith2MessageToSameRecipient() {
        let messageText = "hello!"
        // GIVEN a mailbox with two messages to the same recipient
        let olderMessage = Message.incomingMessage(
            message: .textMessage(message: IncomingMessageData(
                sender: firstTestJournalist,
                messageText: messageText,
                dateReceived: testDate(hourFromNoon: 0)
            ))
        )

        let recentMessage = Message.incomingMessage(
            message: .textMessage(message: IncomingMessageData(
                sender: firstTestJournalist,
                messageText: "how's it going?",
                dateReceived: testDate(hourFromNoon: 1)
            ))
        )

        // WHEN finding the active conversation
        let activeMessage = InboxViewModel.findActiveConversation(in: [olderMessage, recentMessage])

        let expectedActiveMessage = ActiveConversation(
            recipient: firstTestJournalist,
            lastMessageUpdated: testDate(hourFromNoon: 1),
            messages: [olderMessage, recentMessage]
        )

        // THEN the active conversation is the most recent one
        XCTAssertEqual(activeMessage, expectedActiveMessage)
    }

    func testActiveConversationWith2MessageToDifferentRecipients() {
        // GIVEN a mailbox with two messages to different recipient
        let olderMessage = Message.incomingMessage(
            message: .textMessage(message: IncomingMessageData(
                sender: firstTestJournalist,
                messageText: "hello!",
                dateReceived: testDate(hourFromNoon: -1)
            ))
        )

        let recentMessage = Message.incomingMessage(
            message: .textMessage(message: IncomingMessageData(
                sender: secondTestJournalist,
                messageText: "how's it going?",
                dateReceived: testDate(hourFromNoon: 0)
            ))
        )

        // WHEN finding the active conversation
        let activeMessage = InboxViewModel.findActiveConversation(in: [recentMessage, olderMessage])
        let expectedActiveMessage = ActiveConversation(
            recipient: secondTestJournalist,
            lastMessageUpdated: testDate(),
            messages: [recentMessage]
        )
        // THEN the active conversation is the most recent one
        XCTAssertEqual(activeMessage.unsafelyUnwrapped, expectedActiveMessage)
    }

    func testActiveConversationsWith3MessagesIncludingToCurrentUser() throws {
        let messageText = "hello!"

        let privateSendingQueueSecret = try PrivateSendingQueueSecret.fromSecureRandom()
        // GIVEN a mailbox with two messages to different recipient
        let olderMessage = Message.incomingMessage(
            message: .textMessage(
                message: IncomingMessageData(
                    sender: firstTestJournalist,
                    messageText: messageText,
                    dateReceived: testDate(hourFromNoon: 0)
                )
            )
        )

        let recentMessage = Message.incomingMessage(
            message: .textMessage(
                message: IncomingMessageData(
                    sender: secondTestJournalist,
                    messageText: "how's it going?",
                    dateReceived: testDate(hourFromNoon: 1)
                )
            )
        )

        let recentMessageToCurrentUser = Message.outboundMessage(
            message: OutboundMessageData(
                recipient: thirdTestJournalist,
                messageText: messageText,
                dateQueued: testDate(hourFromNoon: 1),
                hint: HintHmac(hint: PrivateSendingQueueHmac.hmac(
                    secretKey: privateSendingQueueSecret.bytes,
                    message: messageText.asBytes()
                ))
            )
        )

        // WHEN finding the active conversation
        let activeMessage = InboxViewModel.findActiveConversation(in: [
            olderMessage,
            recentMessage,
            recentMessageToCurrentUser
        ])

        let expectedActiveMessage = ActiveConversation(
            recipient: secondTestJournalist,
            lastMessageUpdated: testDate(hourFromNoon: 1),
            messages: [recentMessage]
        )

        // THEN the active conversation is the most recent one, that is not to the current user
        XCTAssertEqual(activeMessage, expectedActiveMessage)
    }

    // MARK: Inactive conversations tests

    func testInactiveConversationsWithNoMessages() {
        // GIVEN a mailbox with no messages
        let mailbox = Set<Message>()

        // WHEN when finding the inactive conversation
        let inactiveMessages = InboxViewModel.findInactiveMessages(in: mailbox)

        // THEN no inactive conversations are found
        XCTAssertNil(inactiveMessages)
    }

    func testInactiveConversationWith1Messages() {
        // GIVEN a mailbox with a single message
        let recentMessage = Message.incomingMessage(
            message: .textMessage(
                message: IncomingMessageData(
                    sender: firstTestJournalist,
                    messageText: "hello!",
                    dateReceived: testDate()
                )
            )
        )

        // WHEN finding an inactive conversation
        let inactiveMessages = InboxViewModel.findInactiveMessages(in: [recentMessage])

        // THEN no inactive conversations are found
        XCTAssertNil(inactiveMessages)
    }

    func testInactiveConversationWith2MessagesSameRecipient() {
        // GIVEN a mailbox with two messages to the same recipient
        let olderMessage = Message.incomingMessage(
            message: .textMessage(
                message: IncomingMessageData(
                    sender: firstTestJournalist,
                    messageText: "hello!",
                    dateReceived: testDate()
                )
            )
        )

        let recentMessage = Message.incomingMessage(
            message: .textMessage(
                message: IncomingMessageData(
                    sender: firstTestJournalist,
                    messageText: "how's it going?",
                    dateReceived: testDate(hourFromNoon: 10)
                )
            )
        )

        // WHEN finding the inactive conversation
        let inactiveMessage = InboxViewModel.findInactiveMessages(in: [olderMessage, recentMessage])

        // THEN no inactive conversations should be found
        XCTAssertNil(inactiveMessage)
    }

    func testInActiveConversationWith2MessageToDifferentRecipients() {
        // GIVEN a mailbox with two messages to different recipient
        let olderMessage = Message.incomingMessage(
            message: .textMessage(
                message: IncomingMessageData(
                    sender: firstTestJournalist,
                    messageText: "hello!",
                    dateReceived: testDate(hourFromNoon: 10)
                )
            )
        )

        let recentMessage = Message.incomingMessage(
            message: .textMessage(
                message: IncomingMessageData(
                    sender: secondTestJournalist,
                    messageText: "how's it going?",
                    dateReceived: testDate()
                )
            )
        )

        // WHEN finding the inactive conversation
        let inactiveMessage = InboxViewModel.findInactiveMessages(in: [olderMessage, recentMessage])

        let expectedInactiveMessage = InactiveConversation(recipient: secondTestJournalist, messages: [recentMessage])

        // THEN there should only be one inactive conversation
        XCTAssertEqual(inactiveMessage?.count, 1)

        // and the inactive conversation is the older one
        XCTAssertEqual(inactiveMessage!.first, expectedInactiveMessage)
    }

    func testInActiveConversationsWith3MessagesIncludingToCurrentUser() throws {
        let messageText = "hello!"

        let privateSendingQueueSecret = try PrivateSendingQueueSecret.fromSecureRandom()
        // GIVEN a mailbox with two messages to different recipient
        let olderMessage = Message.incomingMessage(
            message: .textMessage(
                message: IncomingMessageData(
                    sender: firstTestJournalist,
                    messageText: "hello!",
                    dateReceived: testDate(hourFromNoon: 10)
                )
            )
        )

        let recentMessage = Message.incomingMessage(
            message: .textMessage(
                message: IncomingMessageData(
                    sender: secondTestJournalist,
                    messageText: "how's it going?",
                    dateReceived: testDate()
                )
            )
        )

        let recentMessageToCurrentUser = Message.outboundMessage(
            message: OutboundMessageData(
                recipient: thirdTestJournalist,
                messageText: messageText,
                dateQueued: testDate(hourFromNoon: 10),
                hint: HintHmac(hint: PrivateSendingQueueHmac.hmac(
                    secretKey: privateSendingQueueSecret.bytes,
                    message: messageText.asBytes()
                ))
            )
        )

        // WHEN finding the inactive conversation
        let inactiveMessages = InboxViewModel.findInactiveMessages(in: [
            olderMessage,
            recentMessage,
            recentMessageToCurrentUser
        ])

        let expectedInactiveMessage = InactiveConversation(recipient: secondTestJournalist, messages: [recentMessage])

        // THEN the inactive conversation is the older one, and it's not to the current user
        XCTAssertEqual(inactiveMessages!.first, expectedInactiveMessage)
    }

    func testInActiveConversationsWith3Messages() {
        // GIVEN a mailbox with three messages to different recipients
        let olderMessage = Message.incomingMessage(
            message: .textMessage(
                message: IncomingMessageData(
                    sender: firstTestJournalist,
                    messageText: "hello!",
                    dateReceived: testDate()
                )
            )
        )

        let recentMessage = Message.incomingMessage(
            message: .textMessage(
                message: IncomingMessageData(
                    sender: secondTestJournalist,
                    messageText: "how's it going?",
                    dateReceived: testDate(hourFromNoon: 10)
                )
            )
        )

        let recentestMessage = Message.incomingMessage(
            message: .textMessage(
                message: IncomingMessageData(
                    sender: thirdTestJournalist,
                    messageText: "It's going great",
                    dateReceived: testDate(hourFromNoon: 11)
                )
            )
        )

        // WHEN finding the inactive conversation
        let inactiveMessages = InboxViewModel.findInactiveMessages(in: [recentMessage, olderMessage, recentestMessage])

        let expectedInactiveMessage = InactiveConversation(recipient: secondTestJournalist, messages: [recentMessage])
        let expectedSecondInactiveMessage = InactiveConversation(
            recipient: firstTestJournalist,
            messages: [olderMessage]
        )

        // THEN the inactive conversation are the two older ones
        XCTAssertEqual(inactiveMessages!.count, 2)
        XCTAssert(inactiveMessages!.contains(where: { $0 == expectedSecondInactiveMessage }))
        XCTAssert(inactiveMessages!.contains(where: { $0 == expectedInactiveMessage }))
    }
}

// swiftlint:enable force_try
