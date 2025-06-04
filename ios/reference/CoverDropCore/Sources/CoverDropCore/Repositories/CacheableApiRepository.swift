import Foundation

public class CacheableApiRepository<T: Codable> {
    init(
        maxCacheAge: Double,
        now: Date,
        urlSession: URLSession,
        defaultResponse: T? = nil,
        localRepository: LocalCacheFileRepository<T>,
        cacheableWebRepository: any CacheableWebRepository<T>
    ) {
        self.maxCacheAge = maxCacheAge
        self.now = now
        self.urlSession = urlSession
        self.defaultResponse = defaultResponse
        self.localRepository = localRepository
        self.cacheableWebRepository = cacheableWebRepository
    }

    var maxCacheAge: TimeInterval

    public var now: Date
    public var urlSession: URLSession

    public var defaultResponse: T?

    var localRepository: LocalCacheFileRepository<T>
    var cacheableWebRepository: any CacheableWebRepository<T>

    public func downloadAndUpdateAllCaches(cacheEnabled: Bool = true) async throws -> T? {
        // We can provide a default response if the cache or web apis are not available
        let defaultData = defaultResponse

        if !cacheEnabled {
            guard let response = await getFromApiOnly() else {
                return defaultData
            }
            return response
        }

        // Check to see if the local cache file exists
        let hasCache = localRepository.doesCacheExists()

        // This should only run on first ever load.
        if !hasCache {
            return await initCache()
        }

        let shouldDownload = await shouldDownload()
        if shouldDownload {
            _ = await getFromApiAndCache()
        }

        return try await localRepository.load()
    }

    /// This tries to load data from the api and then caches and returns the result.
    /// - Returns: Optional T, the result of the web request or the default data if the request fails
    func getFromApiAndCache() async -> T? {
        do {
            let webData: T = try await cacheableWebRepository.get(params: [:])
            try await localRepository.save(data: webData)
            return webData
        } catch {
            if let defaultResponse {
                try? await localRepository.save(data: defaultResponse)
            }
            return defaultResponse
        }
    }

    func getFromApiOnly() async -> T? {
        if let webData: T = try? await cacheableWebRepository.get(params: [:]) {
            return webData
        }
        return nil
    }

    func initCache() async -> T? {
        guard let response = await getFromApiAndCache() else {
            if let cacheData = defaultResponse {
                try? await localRepository.save(data: cacheData)
            }
            return defaultResponse
        }
        return response
    }

    /// We want to download when we have a cache that is older than the `maxCacheAge`.
    /// In case of errors we err on the side of action and suggest we should download.
    func shouldDownload() async -> Bool {
        if let cacheAge = try? localRepository.getCacheAge(nowOverride: now) {
            return cacheAge > maxCacheAge
        }
        return true
    }

    /// For debugging purposes
    func getTimestampOfCachedFile() async throws -> Date? {
        if !localRepository.doesCacheExists() {
            return nil
        }

        return try localRepository.getCacheLastUpdateDate()
    }
}
