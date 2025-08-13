import Foundation
import SwiftUI

struct InformationView: View {
    enum ViewType {
        case error, info, action
    }

    var viewType: ViewType
    var title: String
    var message: String
    var imagePath: String
    var textColor: Color
    var strokeColor: Color
    var action: () -> Void = {}

    init(viewType: ViewType, title: String, message: String, action: @escaping () -> Void = {}) {
        self.action = action
        self.viewType = viewType
        self.title = title
        self.message = message
        switch viewType {
        case .error:
            imagePath = "exclamationmark.triangle.fill"
            textColor = Color.UserLoginView.errorMessageForegroundColor
            strokeColor = Color.UserLoginView.errorMessageStrokeColor

        case .info:
            imagePath = "info.circle.fill"
            textColor = Color.NewMessageView.messageInformationColor
            strokeColor = Color.NewMessageView.messageInformationStrokeColor

        case .action:
            imagePath = "info.circle.fill"
            textColor = Color.white
            strokeColor = Color.white
        }
    }

    var body: some View {
        VStack(alignment: .leading) {
            Button(action: action) {
                HStack(alignment: .top) {
                    Image(systemName: imagePath)
                        .foregroundColor(strokeColor)
                        .offset(CGSize(width: 0, height: -4))
                    VStack(alignment: .leading) {
                        Text("\(title)")
                            .textStyle(ErrorBoxTitleTextStyle())
                            .foregroundColor(textColor)

                        Text(message).textStyle(BodyStyle())
                            .fixedSize(horizontal: false, vertical: true)
                            .padding([.trailing], Padding.medium)
                            .foregroundColor(textColor)
                    }
                    Spacer()
                    if viewType == .action {
                        VStack(alignment: .center) {
                            Image(systemName: "chevron.forward").resizable().frame(width: 7, height: 11)
                                .padding(.top, 12)
                        }
                    }
                }
            }
        }
        .frame(maxWidth: .infinity)
        .padding([.leading, .trailing, .top], Padding.medium)
        .padding([.bottom], Padding.small)
        .overlay(
            RoundedRectangle(cornerRadius: CornerRadius.small)
                .stroke(strokeColor, style: StrokeStyle(lineWidth: 1))
        )
    }
}
