import Foundation

protocol PublicKeyRepositoryProtocol {
    func loadKeys(cacheEnabled: Bool) async throws -> PublicKeysData
}

/// This repository is for managing public keys published by the Private Key Infrastructure
/// This repository tries to load the keys data from disk, if this fails it will then try and reload the data from the
/// web api
/// and store the results to file before returning them
public class PublicKeyRepository: CacheableApiRepository<PublicKeysData> {
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
