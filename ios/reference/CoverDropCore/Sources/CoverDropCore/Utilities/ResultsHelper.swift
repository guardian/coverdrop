import Foundation

extension Result {
    var isSuccess: Bool {
        if case .success = self {
            return true
        }
        return false
    }
}
