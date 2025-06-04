import Foundation

/// This repository is for managing dead drops published from the API `/users/dead-drops`
/// 1. This repository tries to load the last succesfull dead drop ID from disk, if this fails it will then try and get
/// dead drops from id 0
class DeadDropRepository: CacheableApiRepository<DeadDropData> {
    init(now: Date = DateFunction.currentTime(), config: CoverDropConfig, urlSession: URLSession) {
        super.init(
            maxCacheAge: TimeInterval(Constants.clientDefaultDownloadRateSeconds),
            now: now,
            urlSession: urlSession,
            defaultResponse: DeadDropData(deadDrops: []),
            localRepository: LocalCacheFileRepository<DeadDropData>(
                file: CoverDropFiles.deadDropCache
            ),
            cacheableWebRepository: DeadDropWebRepository(urlSession: urlSession, baseUrl: config.apiBaseUrl)
        )
    }

    /// This loads dead drops from the `/user/dead-drops/` endpoint and caches the response.
    /// Each dead drop will only be kept in cache for 2 weeks, and will be removed during a merge and trim operation,
    /// once the dead drop `createdAt` date  falls outside that period.
    /// A merge and trim only happens when new dead drops have been got from the api.
    /// At any point in time, the dead drop api only has the last 2 weeks worth of dead drops, so inital loading will
    /// generally have a fixed size.

    override func getFromApiAndCache() async -> DeadDropData? {
        do {
            var availableCachedData: DeadDropData = .init(deadDrops: [])

            if let localCachedData = try? await localRepository.load() {
                availableCachedData = localCachedData
            }

            let highestCachedDeadDropId = availableCachedData.deadDrops.max(by: { $0.id < $1.id })?.id ?? 0
            let params = ["ids_greater_than": String(highestCachedDeadDropId)]
            let webData: DeadDropData = try await cacheableWebRepository.get(params: params)

            let mergedDeadDrops = DeadDropRepository.mergeAndTrim(
                existingDeadDrops: availableCachedData,
                newDeadDrops: webData
            )

            try await localRepository.save(data: mergedDeadDrops)
            return mergedDeadDrops
        } catch {
            return defaultResponse
        }
    }

    override func getFromApiOnly() async -> DeadDropData? {
        let params = ["ids_greater_than": "0"]
        if let webData: DeadDropData = try? await cacheableWebRepository.get(params: params) {
            return webData
        } else {
            return nil
        }
    }

    /// Merges two sets of dead drops, with duplicate entries being merged.
    /// Then trims the merged dead drops by `clientDeadDropCacheTtlSeconds` using the `mostRecentTimestamp`
    /// from the most recent dead drops createdAt date.
    /// Any dead drops older than `clientDeadDropCacheTtlSeconds` will be removed
    /// - Parameters:
    ///   - existingDeadDrops: a `DeadDropData` object, normally loaded from a file cache
    ///   - newDeadDrops: a `DeadDropData` object, normally loaded from the dead drop api.
    /// - Returns: The merged and trimmed resulting `DeadDropData`
    static func mergeAndTrim(existingDeadDrops: DeadDropData, newDeadDrops: DeadDropData) -> DeadDropData {
        let deadDropCacheTTL = TimeInterval(Constants.clientDeadDropCacheTtlSeconds)

        let mergedDeadDrops: [DeadDrop] = existingDeadDrops.deadDrops + newDeadDrops.deadDrops

        let uniqueDeadDrops = Set(mergedDeadDrops)

        // identify the newest dead-drop timestamp. We use that as a reference for "now" to avoid
        // using the device clock which might be out-of-sync and could lead to evicting more of
        // fewer items than intended
        guard let mostRecentTimestamp = uniqueDeadDrops.max(by: { $0.createdAt < $1.createdAt }) else {
            return DeadDropData(deadDrops: [])
        }
        let cutOffDate = mostRecentTimestamp.createdAt.date - deadDropCacheTTL
        let mergedAndTrimmedDeadDrops = uniqueDeadDrops.filter { $0.createdAt.date >= cutOffDate }

        return DeadDropData(deadDrops: Array(mergedAndTrimmedDeadDrops))
    }
}
