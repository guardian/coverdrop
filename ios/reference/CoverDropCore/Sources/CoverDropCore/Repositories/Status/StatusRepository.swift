import Foundation
import Sodium

class StatusRepository: CacheableApiRepository<StatusData> {
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
