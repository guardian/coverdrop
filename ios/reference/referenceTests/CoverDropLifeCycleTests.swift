@testable import CoverDropCore
import XCTest

final class CoverDropLifeCycleTests: XCTestCase {
    let config: StaticConfig = .devConfig
    let fileManager = FileManager.default

    // When running these tests in context of a running app, initialisation has already happened at this point
    // So we reset the state and reinitialise again.
    func testInitialization_whenStoragePreviouslyEmpty_thenAppSupportDirExists() async throws {
        let baseUrl = try StorageManager.shared.getBaseDirectoryUrl()

        // delete all files in base directory
        if fileManager.fileExists(atPath: baseUrl.path) {
            for file in fileManager.enumerator(atPath: baseUrl.path)! {
                let filePath = baseUrl.appendingPathComponent(file as! String)
                try fileManager.removeItem(at: filePath)
            }

            // and then the base directory itself
            try fileManager.removeItem(atPath: baseUrl.path)
        }

        XCTAssertFalse(fileManager.fileExists(atPath: baseUrl.path))

        CoverDropService.shared.state = .notInitialized

        guard let task = try? CoverDropService.shared.ensureInitialized(config: config) else {
            XCTFail("Failed to initialize")
            return
        }

        await task.value
        guard case .initialized = CoverDropService.shared.state else {
            XCTFail("Failled to initialize")
            return
        }

        _ = try await CoverDropService.getLibraryBlocking(config: config)

        // check that base directory exists
        XCTAssertTrue(fileManager.fileExists(atPath: baseUrl.path))
    }

    func testInitialization_whenFailedToStart_thenRecoversInReinitialiization() async throws {
        // Check our initial state is correct
        guard case .notInitialized = CoverDropService.shared.state else {
            XCTFail("Not in correct state, expected .notInitialized")
            return
        }

        // Setup the CoverDropService in a failed state
        CoverDropService.shared.state = .failedToInitialize(reason: CoverDropServicesError.notInitialized)

        guard case .failedToInitialize = CoverDropService.shared.state else {
            XCTFail("Not in correct state expected .failedToInitialize")
            return
        }

        // Ensure initialize should recover from .failedToInitialize state
        guard let task = try? CoverDropService.shared.ensureInitialized(config: config) else {
            XCTFail("Failled to initialize")
            return
        }

        await task.value

        if case .initialized = CoverDropService.shared.state {
            XCTAssert(true)
        } else {
            XCTFail("Did not recover from .failedToInitialize state")
        }
    }
}
