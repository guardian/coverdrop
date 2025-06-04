import Foundation

public enum BackgroundMessageSendState {
    private static let PendingKey = "CoverDropBackgroundWorkPending"

    static func writeBackgroundWorkPending(_ result: Bool) {
        UserDefaults.standard.set(result, forKey: BackgroundMessageSendState.PendingKey)
    }

    static func readBackgroundWorkPending() -> Bool? {
        UserDefaults.standard
            .object(forKey: BackgroundMessageSendState.PendingKey) as? Bool
    }

    private static let LastSuccessfulRunTimeKey = "CoverDropBackgroundWorkLastSuccessfulRunTimestamp"

    static func writeBackgroundWorkLastSuccessfulRun(instant: Date) {
        UserDefaults.standard.set(
            instant,
            forKey: BackgroundMessageSendState.LastSuccessfulRunTimeKey
        )
    }

    static func readBackgroundWorkLastSuccessfulRun() -> Date? {
        return UserDefaults.standard
            .object(forKey: BackgroundMessageSendState.LastSuccessfulRunTimeKey) as? Date
    }

    private static let LastTriggerTimeKey = "CoverDropBackgroundWorkLastTriggerTimestamp"

    static func writeBackgroundWorkLastTrigger(instant: Date) {
        UserDefaults.standard.set(
            instant,
            forKey: BackgroundMessageSendState.LastTriggerTimeKey
        )
    }

    static func readBackgroundWorkLastTrigger() -> Date? {
        return UserDefaults.standard
            .object(forKey: BackgroundMessageSendState.LastTriggerTimeKey) as? Date
    }

    static func clearAllState() {
        UserDefaults.standard.removeObject(forKey: BackgroundMessageSendState.PendingKey)
        UserDefaults.standard.removeObject(forKey: BackgroundMessageSendState.LastSuccessfulRunTimeKey)
        UserDefaults.standard.removeObject(forKey: BackgroundMessageSendState.LastTriggerTimeKey)
    }
}
