import Foundation

/**
 * Debug information that can be used to diagnose issues with the app. Typically tucked away at
 * the bottom of the about screen.
 */
public struct DebugContext {
    let lastUpdatePublicKeys: Date?
    let lastUpdateDeadDrops: Date?
    let lastBackgroundTry: Date?
    let lastBackgroundSend: Date?
    let hashedOrgKeys: String?

    public var description: String {
        return """
        public keys: \(prettyDateString(lastUpdatePublicKeys))
        dead drops:  \(prettyDateString(lastUpdateDeadDrops))
        bg success:  \(prettyDateString(lastBackgroundSend))
        bg trigger:  \(prettyDateString(lastBackgroundTry))
        root: \(hashedOrgKeys ?? "none")
        """
    }

    private func prettyDateString(_ date: Date?) -> String {
        guard let date = date else {
            return "never"
        }

        let formatter = DateFormatter()
        formatter.dateFormat = "yyyy-MM-dd+HH:mm:ss"
        formatter.locale = Locale(identifier: "en_UK")
        return formatter.string(from: date)
    }
}
