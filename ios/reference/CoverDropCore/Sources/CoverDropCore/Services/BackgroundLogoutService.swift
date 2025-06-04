import CryptoKit
import Foundation

enum BackgroundLogoutServiceError: Error {
    case missingLastBackgroundDateConfig
}

/// This service measures the time the user has backgrounded the app for, and then logs them out by
/// locking the `secretData` if they are logged in and have exceeded the `maxBackgroundDurationInSeconds`
public enum BackgroundLogoutService {
    public static func logoutIfBackgroundedForTooLong() async throws {
        let lib = try CoverDropService.getLibrary()
        let appConfig = lib.config
        guard let lastBackgroundDate = UserDefaults.standard.object(forKey: "LastBackgroundDate") as? Date else {
            throw BackgroundLogoutServiceError.missingLastBackgroundDateConfig
        }

        let timeIntervalSinceLastBackground = DateFunction.currentTime().timeIntervalSince(lastBackgroundDate)
        if timeIntervalSinceLastBackground > TimeInterval(appConfig.maxBackgroundDurationInSeconds) {
            try await lib.secretDataRepository.lock()
        }
    }
}
