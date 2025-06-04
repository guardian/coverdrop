import Foundation

public enum StatusError: Error {
    case cannotFindFileError
}

/// This helper is used to load the status fixture data from disk for the purpose of unit and UI testing
/// It is located here because our tests are defined across multiple packages, and CoverDropCore is a common dependency
/// of them all
public class StatusDataHelper {
    public func readLocalDataFile() throws -> DeadDropData {
        let data = try readAvailableStatusJson()
        let decodedData = try JSONDecoder().decode(DeadDropData.self, from: data)
        return decodedData
    }

    public func readInitalStatusJson() throws -> Data {
        let name = "001_initial_status"
        guard let resourceUrl = Bundle.module.url(
            forResource: name,
            withExtension: ".json",
            subdirectory: "vectors/set_system_status/system_status"
        ) else { throw DeadDropDateError.cannotFindFileError }
        return try Data(contentsOf: resourceUrl)
    }

    public func readAvailableStatusJson() throws -> Data {
        let name = "002_status_available"
        guard let resourceUrl = Bundle.module.url(
            forResource: name,
            withExtension: ".json",
            subdirectory: "vectors/set_system_status/system_status"
        ) else { throw DeadDropDateError.cannotFindFileError }
        return try Data(contentsOf: resourceUrl)
    }

    public func readUnavailableStatusJson() throws -> Data {
        let name = "003_status_unavailable"
        guard let resourceUrl = Bundle.module.url(
            forResource: name,
            withExtension: ".json",
            subdirectory: "vectors/set_system_status/system_status"
        ) else { throw DeadDropDateError.cannotFindFileError }
        return try Data(contentsOf: resourceUrl)
    }

    public static let shared = StatusDataHelper()

    private init() {
        do {
            _ = try readLocalDataFile()
        } catch {
            // We drop errors sliently to avoid giving away any user interaction in logs
        }
    }
}
