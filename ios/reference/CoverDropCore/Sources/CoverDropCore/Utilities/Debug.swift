import Foundation
import OSLog

public enum Debug {
    static let logger = Logger(
        subsystem: "com.theguardian.reference",
        category: "Debug"
    )

    public static func println(_ message: Any) {
        #if DEBUG
            print(message)
            logger.log("D: \(String(describing: message))")
        #endif
    }
}
