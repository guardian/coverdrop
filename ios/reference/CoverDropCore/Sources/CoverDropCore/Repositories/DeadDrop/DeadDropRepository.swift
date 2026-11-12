import Foundation

/// This repository is for managing dead drops published from the API's `/journalist-to-user/dead-drops` endpoint.
/// The repository loads the cached dead drops from disk and uses the newest `createdAt` of the cached dead drops
/// that can be verified as the `created_after` cursor. If there are no cached dead drops, it gets all dead drops since
/// 1970-01-01.
/// Dead drop timestamps are kept at whole-second precision, which is what the CoverNode signs, and dead drops are
/// deduplicated by signed values only.
class DeadDropRepository: CacheableApiRepository<DeadDropData> {
    /// - Parameter now: only overridden in tests, to fake the current time so the superclass's cache-age check
    ///   (`shouldDownload()`) can be exercised without waiting for the cache to age. The default is evaluated
    ///   once per instance, so create a new repository per use rather than keeping one around.
    init(
        now: Date = DateFunction.currentTime(),
        config: CoverDropConfig,
        urlSession: URLSession
    ) {
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

    /// Verified keys are needed to pull dead drops from the API, so callers must use
    /// `downloadAndUpdateAllCaches(cacheEnabled:verifiedKeys:)` instead.
    @available(*, unavailable, message: "Use downloadAndUpdateAllCaches(cacheEnabled:verifiedKeys:)")
    override func downloadAndUpdateAllCaches(cacheEnabled _: Bool = true) async throws -> DeadDropData? {
        fatalError("unavailable")
    }

    @available(*, unavailable, message: "Use getFromApiAndCache(verifiedKeys:)")
    override func getFromApiAndCache() async -> DeadDropData? {
        fatalError("unavailable")
    }

    /// See `CacheableApiRepository.downloadAndUpdateAllCaches(cacheEnabled:)`. The verified keys are only used
    /// when dead drops are pulled from the API, to establish the `created_after` cursor.
    func downloadAndUpdateAllCaches(
        cacheEnabled: Bool = true,
        verifiedKeys: VerifiedPublicKeys?
    ) async throws -> DeadDropData? {
        try await downloadAndUpdateAllCaches(
            cacheEnabled: cacheEnabled,
            fromApiOnly: { await self.getFromApiOnly() },
            fromApiAndCache: { await self.getFromApiAndCache(verifiedKeys: verifiedKeys) }
        )
    }

    /// This loads dead drops from the `/journalist-to-user/dead-drops/` endpoint and caches the response.
    /// Each dead drop will only be kept in cache for 2 weeks, and will be removed during a merge and trim operation,
    /// once the dead drop `createdAt` date  falls outside that period.
    /// A merge and trim only happens when new dead drops have been fetched from the api.
    /// At any point in time, the dead drop api only has the last 2 weeks worth of dead drops, so inital loading will
    /// generally have a fixed size.
    func getFromApiAndCache(verifiedKeys: VerifiedPublicKeys?) async -> DeadDropData? {
        // Without verified keys we cannot establish a trustworthy cursor, so we do not pull. The next attempt
        // happens once the keys are available.
        guard let verifiedKeys else {
            return (try? await localRepository.load()) ?? defaultResponse
        }

        do {
            var availableCachedData: DeadDropData = .init(deadDrops: [])

            if let localCachedData = try? await localRepository.load() {
                // Dead drops cached by earlier app versions can carry sub-second precision or duplicates
                availableCachedData = DeadDropRepository.normalizeAndDeduplicate(localCachedData)
            }

            // The cache can contain dead drops that do not verify, for example ones with a made-up `createdAt` served
            // by a malicious API. Only the verified ones are allowed to move the cursor.
            let verifiedCachedDeadDrops = VerifiedDeadDrops.fromAllDeadDropData(
                deadDrops: availableCachedData,
                verifiedKeys: verifiedKeys
            )
            let cursor = verifiedCachedDeadDrops.deadDrops
                .map { RFC3339DateTimeString(date: $0.publishedDate) }
                .max() ?? RFC3339DateTimeString(date: Date(timeIntervalSince1970: 0))
            let params = ["created_after": cursor.rfc3339SecondString]
            let webData: DeadDropData = try await cacheableWebRepository.get(params: params)

            let filteredWebData = DeadDropRepository.discardDeadDropsAtOrBeforeCursor(
                deadDrops: DeadDropRepository.normalizeAndDeduplicate(webData),
                cursor: cursor
            )

            let mergedDeadDrops = DeadDropRepository.mergeAndTrim(
                existingDeadDrops: availableCachedData,
                newDeadDrops: filteredWebData,
                trimReference: cursor
            )

            try await localRepository.save(data: mergedDeadDrops)
            return mergedDeadDrops
        } catch {
            return defaultResponse
        }
    }

    override func getFromApiOnly() async -> DeadDropData? {
        let minimumDate = Date(timeIntervalSince1970: 0).rfc3339SecondString
        let params = ["created_after": minimumDate]
        if let webData: DeadDropData = try? await cacheableWebRepository.get(params: params) {
            return DeadDropRepository.normalizeAndDeduplicate(webData)
        } else {
            return nil
        }
    }

    /// Truncates every `createdAt` to whole seconds, the precision that is signed by the CoverNode, and removes
    /// duplicates (see `DeadDrop.==`). The first occurrence of a duplicate is kept and the order is preserved.
    static func normalizeAndDeduplicate(_ deadDropData: DeadDropData) -> DeadDropData {
        var seen = Set<DeadDrop>()
        let uniqueDeadDrops = deadDropData.deadDrops.compactMap { deadDrop -> DeadDrop? in
            var normalized = deadDrop
            normalized.createdAt = deadDrop.createdAt.truncatedToSignedPrecision
            return seen.insert(normalized).inserted ? normalized : nil
        }

        return DeadDropData(deadDrops: uniqueDeadDrops)
    }

    /// A malicious API could replay dead drops the client has already processed by serving them again
    /// at or before the requested cursor, so we discard any that don't satisfy `createdAt > cursor`.
    static func discardDeadDropsAtOrBeforeCursor(deadDrops: DeadDropData,
                                                 cursor: RFC3339DateTimeString) -> DeadDropData {
        let filteredDeadDrops = deadDrops.deadDrops.filter { deadDrop in
            deadDrop.createdAt > cursor
        }

        return DeadDropData(deadDrops: filteredDeadDrops)
    }

    /// Merges two sets of dead drops, with duplicate entries being merged.
    /// Then trims the merged dead drops by `clientDeadDropCacheTtlSeconds` using the `trimReference`.
    /// Any dead drops older than `clientDeadDropCacheTtlSeconds` before the `trimReference` will be removed
    /// - Parameters:
    ///   - existingDeadDrops: a `DeadDropData` object, normally loaded from a file cache
    ///   - newDeadDrops: a `DeadDropData` object, normally loaded from the dead drop api.
    ///   - trimReference: the `createdAt` of the newest dead drop that verifies. We use that as a reference for "now"
    ///   to avoid using the device clock which might be out-of-sync and could lead to evicting more of fewer items than
    ///   intended. It must not come from a dead drop that does not verify, otherwise a malicious API could serve one
    ///   with a far-future `createdAt` and make us evict all genuine dead drops.
    /// - Returns: The merged and trimmed resulting `DeadDropData`
    static func mergeAndTrim(
        existingDeadDrops: DeadDropData,
        newDeadDrops: DeadDropData,
        trimReference: RFC3339DateTimeString
    ) -> DeadDropData {
        let deadDropCacheTTL = TimeInterval(Constants.clientDeadDropCacheTtlSeconds)

        let uniqueDeadDrops = normalizeAndDeduplicate(
            DeadDropData(deadDrops: existingDeadDrops.deadDrops + newDeadDrops.deadDrops)
        ).deadDrops

        let cutOffDate = trimReference.date - deadDropCacheTTL
        let mergedAndTrimmedDeadDrops = uniqueDeadDrops.filter { $0.createdAt.date >= cutOffDate }

        return DeadDropData(deadDrops: mergedAndTrimmedDeadDrops)
    }
}
