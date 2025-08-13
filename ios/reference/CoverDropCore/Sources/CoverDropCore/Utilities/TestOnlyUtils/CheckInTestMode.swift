import Foundation

public extension ProcessInfo {
    var isRunningXCTest: Bool {
        return ProcessInfo.processInfo.processName == "xctest"
    }
}
