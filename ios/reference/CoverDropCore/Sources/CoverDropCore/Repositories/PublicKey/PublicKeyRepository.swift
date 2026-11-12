import Foundation

protocol PublicKeyRepositoryProtocol {
    func loadKeys(cacheEnabled: Bool) async throws -> PublicKeysData
}

/// This repository is for managing public keys published by the Private Key Infrastructure
/// This repository tries to load the keys data from disk, if this fails it will then try and reload the data from the
/// web api
/// and store the results to file before returning them
public class PublicKeyRepository: CacheableApiRepository<PublicKeysData> {
    /// - Parameter now: only overridden in tests, to fake the current time so the superclass's cache-age check
    ///   (`shouldDownload()`) can be exercised without waiting for the cache to age. The default is evaluated
    ///   once per instance, so create a new repository per use rather than keeping one around.
    init(now: Date = DateFunction.currentTime(), config: CoverDropConfig, urlSession: URLSession) {
        super.init(
            maxCacheAge: TimeInterval(Constants.clientDefaultDownloadRateSeconds),
            now: now,
            urlSession: urlSession,
            localRepository: LocalCacheFileRepository<PublicKeysData>(
                file: CoverDropFiles.publicKeysCache
            ),
            cacheableWebRepository: PublicKeyWebRepository(config: config, urlSession: urlSession)
        )
    }
}
