import Foundation
import SwiftUI

struct MessageLengthProgressView: View {
    var messageLengthProgressPercentage: Result<Double, MessageComposeError>

    var body: some View {
        return VStack(alignment: .leading) {
            switch messageLengthProgressPercentage {
            case let .success(percentage):
                ProgressView(value: percentage, total: 100)
                    .progressViewStyle(LinearProgressViewStyle(tint: Color.ProgressBarStyle.fillingColor))
            case let .failure(errorType):
                switch errorType {
                case .invalidCharacter:
                    Label("You've entered an invalid character", systemImage: "exclamationmark.triangle.fill")
                        .labelStyle(MessageTextErrorMessage())
                case .textTooLong:
                    VStack(alignment: .leading) {
                        Label("Message limit reached", systemImage: "exclamationmark.triangle.fill")
                            .labelStyle(MessageTextErrorMessage())
                        Text("Please shorten your message")
                            .textStyle(BodyStyle())
                            .padding(.leading, 28)
                    }
                    ProgressView(value: 100, total: 100)
                        .progressViewStyle(LinearProgressViewStyle(tint: Color.ProgressBarStyle.fullColor))
                case .compressionFailed, .unknownError:
                    Label("Message error, please try again later", systemImage: "exclamationmark.triangle.fill")
                        .labelStyle(MessageTextErrorMessage())
                }
            }
        }.foregroundColor(Color.JournalistNewMessageView.messageListForegroundColor)
    }
}
