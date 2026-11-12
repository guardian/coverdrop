import Foundation
import Sodium

class StatusRepository: CacheableApiRepository<StatusData> {
    /// - Parameter now: only overridden in tests, to fake the current time so the superclass's cache-age check
    ///   (`shouldDownload()`) can be exercised without waiting for the cache to age. The default is evaluated
    ///   once per instance, so create a new repository per use rather than keeping one around.
    init(now: Date = DateFunction.currentTime(), config: CoverDropConfig, urlSession: URLSession) {
        super.init(
            maxCacheAge: TimeInterval(Constants.clientStatusDownloadRateSeconds),
            now: now,
            urlSession: urlSession,
            defaultResponse: StatusData(
                status: .available,
                description: "Available",
                timestamp: RFC3339DateTimeString(
                    date: DateFunction.currentTime()
                ),
                isAvailable: true
            ),
            localRepository: LocalCacheFileRepository<StatusData>(
                file: CoverDropFiles.statusCache
            ),
            cacheableWebRepository: StatusWebRepository(urlSession: urlSession, baseUrl: config.apiBaseUrl)
        )
    }
}
