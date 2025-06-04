import CoverDropCore
import CryptoKit
import Foundation
import SwiftUI

@MainActor
struct MessageView: View {
    var message: UiMessage
    var id: Int

    public init(message: UiMessage, id: Int) {
        self.message = message
        self.id = id
    }

    var body: some View {
        HStack(alignment: .bottom) {
            if message.isOutgoing {
                // Add a spacer so that messages send from this device appear on the right
                Spacer().frame(minWidth: 30, maxWidth: 40)
            }

            VStack {
                if message.isOutgoing {
                    HStack {
                        Spacer()
                        Text("You").textStyle(BodyStyle()).bold()
                    }
                }
                VStack(alignment: .leading) {
                    Text(message.messageText).textStyle(BodyStyle())
                        .padding([.top, .leading, .trailing], Padding.medium)
                    customDivider()
                    HStack(alignment: .top) {
                        switch message.expiryState {
                        case .expired:
                            Text("\(Image(systemName: "exclamationmark.triangle.fill")) Expired")
                                .textStyle(ExpiringMessageMetadata())
                                .padding([.leading], Padding.medium)
                                .accessibilityIdentifier("Expired")
                        case let .soonToBeExpired(expiryCountdownString):
                            if let expiryCountdownString = expiryCountdownString {
                                Text("\(Image(systemName: "info.circle.fill")) Expiring in \(expiryCountdownString)")
                                    .textStyle(ExpiringMessageMetadata())
                                    .padding([.leading], Padding.medium)
                                    .accessibilityIdentifier("Expiring in \(expiryCountdownString)")
                            }
                        case .fresh:
                            switch message {
                            case .incoming:
                                Spacer()
                            case let .outgoing(_, _, _, isPending):
                                if isPending {
                                    Text("\(Image(systemName: "clock.fill")) Pending")
                                        .textStyle(PendingMessageMetadata())
                                        .padding([.leading], Padding.medium)
                                        .accessibilityIdentifier("Pending")
                                } else {
                                    Text("\(Image(systemName: "checkmark.circle.fill")) Sent")
                                        .textStyle(SentMessageMetadata())
                                        .padding([.leading], Padding.medium)
                                        .accessibilityIdentifier("Sent")
                                }
                            }
                        }
                        Spacer()
                        Text("\(message.date, formatter: ConversationViewModel.messageDateFormat)")
                            .textStyle(MessageMetadata())
                            .padding([.trailing], Padding.medium)
                    }
                }
                .background(
                    message.isOutgoing ?
                        Color.JournalistNewMessageView.messageViewCurrentUserColor :
                        Color.JournalistNewMessageView.messageViewUnselectedUserColor
                )
                .cornerRadius(10)
                .padding([.bottom], Padding.medium)
            }
            .id(id)

            if message.isIncoming {
                // if we are not the current user put a spacer on the right
                Spacer()
            }
        }
    }
}

struct MessageView_Previews: PreviewProvider {
    @MainActor struct Container: View {
        @State var incomingExpired = UiMessage.incoming(
            messageText: "hey expired",
            dateReceived: Date(timeIntervalSinceNow: -TimeInterval(3600 * 24 * 15)),
            state: .expired
        )

        @State var incomingExpiring = UiMessage.incoming(
            messageText: "hey slowly expiring",
            dateReceived: Date(timeIntervalSinceNow: -TimeInterval(3600 * 24 * 13)),
            state: .soonToBeExpired(expiryCountdownString: "24h")
        )

        @State var incomingFresh = UiMessage.incoming(
            messageText: "hey non expired",
            dateReceived: Date(timeIntervalSinceNow: -TimeInterval(3600 * 24 * 2)),
            state: .fresh
        )

        @State var outgoingSent = UiMessage.outgoing(
            messageText: "hi",
            dateQueued: Date(timeIntervalSinceNow: -TimeInterval(60 * 60 * 24 * 3)),
            state: .fresh,
            isPending: false
        )

        @State var outgoingPending = UiMessage.outgoing(
            messageText: "hi",
            dateQueued: Date(timeIntervalSinceNow: -TimeInterval(60 * 60 * 24 * 1)),
            state: .fresh,
            isPending: true
        )

        @MainActor var body: some View {
            MessageView(message: incomingExpired, id: 1)
            MessageView(message: incomingExpiring, id: 2)
            MessageView(message: incomingFresh, id: 3)
            MessageView(message: outgoingSent, id: 3)
            MessageView(message: outgoingPending, id: 3)
        }
    }

    static var previews: some View {
        Container()
    }
}
