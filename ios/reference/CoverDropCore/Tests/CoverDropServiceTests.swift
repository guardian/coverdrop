@testable import CoverDropCore
import XCTest

final class CoverDropServiceTests: XCTestCase {
    let config: StaticConfig = .devConfig
    let fileManager = FileManager.default

    /// Before any run, clean all data from storage
    override func setUpWithError() throws {
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
    }

    func testInitialization_whenStoragePreviouslyEmpty_thenAppSupportDirExists() async throws {
        // check that there is not base directory
        let baseUrl = try StorageManager.shared.getBaseDirectoryUrl()
        XCTAssertFalse(fileManager.fileExists(atPath: baseUrl.path))

        // call on app start
        try CoverDropService.shared.ensureInitialized(config: config)
        _ = try await CoverDropService.getLibraryBlocking()

        // check that base directory exists
        XCTAssertTrue(fileManager.fileExists(atPath: baseUrl.path))

        // check that by default the other files do not exists
        for file in CoverDropFiles.allCases {
            let fileUrl = try StorageManager.shared.getFullUrl(file: file)
            XCTAssertFalse(fileManager.fileExists(atPath: fileUrl.path))
        }
    }
}
