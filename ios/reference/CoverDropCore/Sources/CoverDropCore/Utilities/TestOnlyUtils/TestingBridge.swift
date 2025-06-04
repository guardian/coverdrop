import Foundation

/// All testing flags that e.g. UI tests might pas to our program to change its default behavior
public enum TestingFlag: String {
    case startWithEmptyStorage = "START_WITH_EMPTY_STORAGE"
    case startWithNonEmptyStorage = "START_WITH_NON_EMPTY_STORAGE"
    case removeBackgroundSendStateOnStart = "REMOVE_BACKGROUND_STATE"
    case disableAnimations = "DISABLE_ANIMATIONS"
    case mockedDataEmptyKeysData = "EMPTY_KEYS_DATA"
    case mockedDataMultipleJournalists = "MULTIPLE_JOURNALIST_SCENARIO"
    case mockedDataNoDefaultJournalist = "MOCKED_DATA_NO_DEFAULT_JOURNALIST"
    case mockedDataStatusUnavailable = "STATUS_UNAVAILABLE"
    case mockedDataExpiredMessagesScenario = "EXPIRED_MESSAGES_SCENARIO"
    case forceSingleRecipient = "FORCE_SINGLE_RECIPIENT"
}

public enum TestingBridge {
    /// Returns `true` if the given testing flag has been enabled for the reference application
    public static func isEnabled(_ flag: TestingFlag, processInfo: ProcessInfo? = nil) -> Bool {
        let processInfo = processInfo ?? ProcessInfo.processInfo
        return processInfo.arguments.contains(flag.rawValue)
    }

    public static func setTestingFlags(launchArguments: inout [String], flags: [TestingFlag]) {
        for flag in flags {
            launchArguments.append(flag.rawValue)
        }
    }

    static var currentTimeOverride: Date?

    static func setCurrentTimeOverride(override: Date?) {
        currentTimeOverride = override
    }

    public static func getCurrentTimeOverride() -> Date? {
        return currentTimeOverride
    }

    /// Returns `true` if the reference app should enable mocked API resonses
    public static func isMockedDataEnabled(config: CoverDropConfig) -> Bool {
        #if DEBUG
            // We only want to mock data if we are in dev mode
            // This allows local development against production infrastructure by changing the env type
            // and also allows local development of the iOS Live app against prod without overriding all network
            // requests
            if config.envType == .dev {
                return true
            } else {
                return false
            }
        #else
            return false
        #endif
    }
}
