import Combine
import Foundation
import SwiftUI

extension Publishers {
    static var isKeyboardShown: AnyPublisher<Bool, Never> {
        let willShow = NotificationCenter.default.publisher(for: UIResponder.keyboardWillShowNotification)
            .map { notification in
                (notification.userInfo?[UIResponder.keyboardFrameEndUserInfoKey] as? CGRect)?.height ?? 0 > 0
            }
        let willHide = NotificationCenter.default.publisher(for: UIResponder.keyboardWillHideNotification)
            .map { _ in false }
        return MergeMany(willShow, willHide)
            .eraseToAnyPublisher()
    }
}
