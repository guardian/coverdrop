import Foundation

class LocalCacheFileRepository<T: Codable> {
    private var file: CoverDropFiles

    init(file: CoverDropFiles) {
        self.file = file
    }

    /// Wether the cache file exists on disk.
    /// - Returns: `true` if the cache file exists, `false` otherwise.
    func doesCacheExists() -> Bool {
        return StorageManager.shared.doesFileExist(file: file)
    }

    /// The age of the cache file as a `TimeInterval`. Throws if it does not exist.
    /// - Returns: The age of the cache file
    /// - Throws: `StorageManagerError.fileNotFound`
    func getCacheAge(nowOverride: Date? = nil) throws -> TimeInterval {
        return try StorageManager.shared.getFileAge(file: file, nowOverride: nowOverride)
    }

    func getCacheLastUpdateDate() throws -> Date {
        return try StorageManager.shared.getLastModifiedDate(file: file)
    }

    /// Loads the cache file from disk and decodes to T
    /// - Returns: T representation of the cached dead drop json file
    /// - Throws: If decoding fails or file is not available
    func load() async throws -> T {
        let data = try StorageManager.shared.readFile(file: file)
        let jsonData = Data(data)
        return try JSONDecoder().decode(T.self, from: jsonData)
    }

    /// Saves the supplied data to the local cache file
    /// - Parameter T: T data to be cached.
    /// - Throws: if writing to the output file fails or JSON encoding fails
    func save(data: T) async throws {
        let jsonData = try JSONEncoder().encode(data)
        try StorageManager.shared.writeFile(file: file, data: Array(jsonData))
    }
}
