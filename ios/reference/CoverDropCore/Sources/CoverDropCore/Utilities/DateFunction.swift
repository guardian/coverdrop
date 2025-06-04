import Foundation

public enum DateError: Error {
    case dateArithmeticFailed
}

public enum DateFunction {
    public static func currentKeysPublishedTime() -> Date {
        var date = Date()
        #if DEBUG
            do {
                if let generatedAtDate = try PublicKeysHelper.readLocalGeneratedAtFile() {
                    date = generatedAtDate
                }
            } catch { Debug.println("Failed to get local keys generated file") }
        #endif
        return date
    }

    public static func currentTime() -> Date {
        #if DEBUG
            if TestingBridge.isEnabled(.mockedDataExpiredMessagesScenario) {
                let keysDate = DateFunction.currentKeysPublishedTime()
                return Date(timeInterval: -TimeInterval(60 * 60 * 24 * 13), since: keysDate)
            }
            if let override = TestingBridge.getCurrentTimeOverride() {
                return override
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
