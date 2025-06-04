import Foundation

public enum DeadDropDateError: Error {
    case cannotFindFileError
}

/// This helper is used to load the dead drop fixture data from disk for the purpose of unit and UI testing
/// It is located here because our tests are defined across multiple packages, and CoverDropCore is a common dependency
/// of them all
public class DeadDropDataHelper {
    public func readLocalDataFile() throws -> DeadDropData {
        let data = try readLoadDeadDropJson()
        let decodedData = try JSONDecoder().decode(DeadDropData.self, from: data)
        return decodedData
    }

    public func readLoadDeadDropJson() throws -> Data {
        let name = "003_journalist_replied_and_processed"
        guard let resourceUrl = Bundle.module.url(
            forResource: name,
            withExtension: ".json",
            subdirectory: "vectors/messaging_scenario/user_dead_drops"
        ) else { throw DeadDropDateError.cannotFindFileError }
        return try Data(contentsOf: resourceUrl)
    }

    public func readLoadMultipleJournalistDeadDropJson() throws -> Data {
        let name = "004_journalist_2_replied_and_processed"
        guard let resourceUrl = Bundle.module.url(
            forResource: name,
            withExtension: ".json",
            subdirectory: "vectors/multiple_journalists_messaging_scenario/user_dead_drops"
        ) else { throw DeadDropDateError.cannotFindFileError }
        return try Data(contentsOf: resourceUrl)
    }

    public static let shared = DeadDropDataHelper()

    private init() {
        do {
            _ = try readLocalDataFile()
        } catch {
            // We drop errors sliently to avoid giving away any user interaction in logs
        }
    }
}
