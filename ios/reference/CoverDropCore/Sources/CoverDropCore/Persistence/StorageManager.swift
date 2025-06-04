import Foundation

enum StorageManagerError: Error {
    case fileNotFound
    case unsupportedFileProtection
}

/// The class manages all CoverDrop related files (see `CoverDropFiles`).
class StorageManager {
    static let shared = StorageManager()

    private init() {}

    /// To be called on app start. Ensures that the base directory exists.
    /// Also ensures that the permissions are correct.
    /// In later version this might also execute some migrations if needed.
    public func onAppStart() throws {
        try ensureBaseDirectoryExists()

        for file in CoverDropFiles.allCases {
            if !doesFileExist(file: file) {
                continue
            }

            try ensureCorrectFilePermissions(file: file)
        }
    }

    /// This touches the file to update its created and last-modified date. Throws `StorageManagerError.fileNotFound`
    /// if the file does not exist.
    public func touchFile(file: CoverDropFiles) throws {
        try setCreatedAtDate(file: file, now: Date())
        try setLastModifiedDate(file: file, now: Date())
    }

    /// Returns `true` if the file exists, `false` otherwise.
    public func doesFileExist(file: CoverDropFiles) -> Bool {
        guard let url = try? getFullUrl(file: file) else {
            return false
        }
        return FileManager.default.fileExists(atPath: url.path)
    }

    /// Returns the age of the file as a `TimeInterval`. If the file does not exist, it throws
    /// `StorageManagerError.fileNotFound`.
    public func getFileAge(file: CoverDropFiles, nowOverride: Date? = nil) throws -> TimeInterval {
        let lastModifiedDate = try getLastModifiedDate(file: file)
        let now = nowOverride ?? DateFunction.currentTime()
        return now.timeIntervalSince(lastModifiedDate)
    }

    /// Returns the content of the given file.
    /// If the file does not exist, it throws `StorageManagerError.fileNotFound`.
    public func readFile(file: CoverDropFiles) throws -> [UInt8] {
        let url = try getFullUrl(file: file)

        if !FileManager.default.fileExists(atPath: url.path) {
            throw StorageManagerError.fileNotFound
        }

        let data = try Data(contentsOf: url)
        return Array(data)
    }

    /// Writes the given data to the file. If the file does not exist, it will be created.
    public func writeFile(file: CoverDropFiles, data: [UInt8]) throws {
        let url = try getFullUrl(file: file)

        var options: Data.WritingOptions = []
        try options.insert(file.getFileProtectionMode().toDataWritingOptions())

        let data = Data(data)
        try data.write(to: url, options: options)

        try ensureCorrectFilePermissions(file: file)
    }

    // MARK: - Internal and testing methods below

    func ensureBaseDirectoryExists() throws {
        let url = try getBaseDirectoryUrl()
        let path = url.path
        if !FileManager.default.fileExists(atPath: path) {
            try FileManager.default.createDirectory(
                atPath: path,
                withIntermediateDirectories: true,
                attributes: nil
            )
        }
    }

    func ensureCorrectFilePermissions(file: CoverDropFiles) throws {
        var url = try getFullUrl(file: file)
        let path = url.path

        // If the file does not exist, no need to set permissions
        if !FileManager.default.fileExists(atPath: path) {
            return
        }

        // File protection options can cause issues accessing files
        // when we are in the background, so we make sure they are not set
        try? FileManager.default.setAttributes(
            [.protectionKey: file.getFileProtectionMode()],
            ofItemAtPath: path
        )

        // Exclude from backups
        var res = URLResourceValues()
        res.isExcludedFromBackup = true
        try url.setResourceValues(res)
    }

    /// This returns the created-at date of the file.
    /// If the file does not exist, it throws `StorageManagerError.fileNotFound`.
    func getCreatedAtDate(file: CoverDropFiles) throws -> Date {
        let url = try getFullUrl(file: file)

        let attributes: [FileAttributeKey: Any] =
            try FileManager.default.attributesOfItem(atPath: url.path)
        let optionalCreationDate = attributes[FileAttributeKey.creationDate] as? Date
        guard let creationDate = optionalCreationDate else {
            throw StorageManagerError.fileNotFound
        }
        return creationDate
    }

    /// This returns the last-modified date of the file.
    /// If the file does not exist, it throws `StorageManagerError.fileNotFound`.
    func getLastModifiedDate(file: CoverDropFiles) throws -> Date {
        let url = try getFullUrl(file: file)

        let attributes: [FileAttributeKey: Any] =
            try FileManager.default.attributesOfItem(atPath: url.path)
        let optionalModificationDate = attributes[FileAttributeKey.modificationDate] as? Date
        guard let modificationDate = optionalModificationDate else {
            throw StorageManagerError.fileNotFound
        }
        return modificationDate
    }

    func getBaseDirectoryUrl(create: Bool = true) throws -> URL {
        return try FileManager.default.url(
            for: .applicationSupportDirectory,
            in: .userDomainMask,
            appropriateFor: nil,
            create: create
        )
    }

    func getFullUrl(file: CoverDropFiles) throws -> URL {
        let baseUrl = try getBaseDirectoryUrl()
        let url = baseUrl.appending(
            path: file.rawValue,
            directoryHint: .notDirectory
        )
        return url
    }

    func setCreatedAtDate(file: CoverDropFiles, now: Date) throws {
        let url = try getFullUrl(file: file)
        let attributes = [FileAttributeKey.creationDate: now]
        try FileManager.default.setAttributes(
            attributes,
            ofItemAtPath: url.path
        )
    }

    func setLastModifiedDate(file: CoverDropFiles, now: Date) throws {
        let url = try getFullUrl(file: file)
        let attributes = [FileAttributeKey.modificationDate: now]
        try FileManager.default.setAttributes(
            attributes,
            ofItemAtPath: url.path
        )
    }

    func deleteFile(file: CoverDropFiles) throws {
        if !doesFileExist(file: file) {
            return
        }

        let url = try getFullUrl(file: file)
        try FileManager.default.removeItem(atPath: url.path)
    }
}
