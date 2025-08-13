import Foundation

public enum DateError: Error {
    case dateArithmeticFailed
}

public enum DateFunction {
    public static func currentTime() -> Date {
        #if DEBUG
            if let override = TestingBridge.getCurrentTimeOverride() {
                return override
            }
            if ProcessInfo.processInfo.isRunningXCTest {
                return CoverDropServiceHelper.currentTimeForKeyVerification()
            }
        #endif
        return Date()
    }
}

public extension Date {
    func plusSeconds(_ seconds: Int) throws -> Date {
        guard let result = Calendar.current.date(
            byAdding: .second,
            value: seconds,
            to: self
        ) else {
            throw DateError.dateArithmeticFailed
        }
        return result
    }

    func minusSeconds(_ seconds: Int) throws -> Date {
        return try plusSeconds(-seconds)
    }
}
