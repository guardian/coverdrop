@testable import CoverDropCore
import XCTest

final class StorageManagerTests: XCTestCase {
    var storageManager = StorageManager.shared
    var testFilePublic = CoverDropFiles.deadDropCache
    var testFileSecret = CoverDropFiles.encryptedStorage

    override func tearDown() {
        try? storageManager.deleteFile(file: testFilePublic)
        try? storageManager.deleteFile(file: testFileSecret)
        super.tearDown()
    }

    func testOnAppStart_whenCalled_thenBaseDirectoryExists() throws {
        let baseDirectory = try storageManager.getBaseDirectoryUrl(create: false)
        XCTAssertTrue(FileManager.default.fileExists(atPath: baseDirectory.path))
    }

    func testOnTouch_whenFileExists_thenLastModifiedUpdated() throws {
        try storageManager.writeFile(file: testFilePublic, data: [])

        // back date the created-at and last-modified timestamp
        let hour = 60.0 * 60.0
        let backDate = Date(timeIntervalSinceNow: -hour)
        try storageManager.setCreatedAtDate(file: testFilePublic, now: backDate)
        try storageManager.setLastModifiedDate(file: testFilePublic, now: backDate)
        let age = try storageManager.getFileAge(file: testFilePublic)
        XCTAssertEqual(age, hour, accuracy: 1.0)
        XCTAssertGreaterThan(age, 10.0)

        // call touch and the created and last-modified timestamp should be updated
        try storageManager.touchFile(file: testFilePublic)
        let newAge = try storageManager.getFileAge(file: testFilePublic)
        XCTAssertLessThan(newAge, 10.0)

        let createdAt = try storageManager.getCreatedAtDate(file: testFilePublic)
        let lastModified = try storageManager.getLastModifiedDate(file: testFilePublic)
        XCTAssertGreaterThan(createdAt, backDate)
        XCTAssertGreaterThan(lastModified, backDate)
    }

    func testWriteReadDeleteFile() throws {
        let testData: [UInt8] = [1, 2, 3, 4, 5]
        try storageManager.writeFile(file: testFilePublic, data: testData)

        let readData = try storageManager.readFile(file: testFilePublic)
        XCTAssertEqual(testData, readData)

        try storageManager.deleteFile(file: testFilePublic)
        XCTAssertFalse(storageManager.doesFileExist(file: testFilePublic))
        XCTAssertThrowsError(try storageManager.readFile(file: testFilePublic)) { error in
            XCTAssertEqual(error as? StorageManagerError, .fileNotFound)
        }
    }

    func testFilePermissions_forPublicFile() throws {
        try storageManager.writeFile(file: testFilePublic, data: [])
        let url = try storageManager.getFullUrl(file: testFilePublic)
        let attributes = try FileManager.default.attributesOfItem(atPath: url.path)

        /// Check that file permissions are set to `0x644 (rw-r--r--)` (defaults)
        let permissions = attributes[.posixPermissions] as? NSNumber
        XCTAssertEqual(permissions?.intValue, 0o644)

        // Unfortunately, the `attributes` do not contain the `.protectionKey` value

        // Check that it is excluded from backups
        let resourceValues = try url.resourceValues(forKeys: [.isExcludedFromBackupKey])
        XCTAssertTrue(resourceValues.isExcludedFromBackup!)
    }

    func testFilePermissions_forSecretFile() throws {
        try storageManager.writeFile(file: testFileSecret, data: [])
        let url = try storageManager.getFullUrl(file: testFileSecret)
        let attributes = try FileManager.default.attributesOfItem(atPath: url.path)

        /// Check that file permissions are set to `0x644 (rw-r--r--)` (defaults)
        let permissions = attributes[.posixPermissions] as? NSNumber
        XCTAssertEqual(permissions?.intValue, 0o644)

        // Unfortunately, the `attributes` do not contain the `.protectionKey` value

        // Check that it is excluded from backups
        let resourceValues = try url.resourceValues(forKeys: [.isExcludedFromBackupKey])
        XCTAssertTrue(resourceValues.isExcludedFromBackup!)
    }
}
