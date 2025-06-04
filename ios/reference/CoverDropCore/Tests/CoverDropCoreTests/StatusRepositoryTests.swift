@testable import CoverDropCore
import Sodium
import XCTest

final class StatusRepositoryTests: XCTestCase {
    let config: StaticConfig = .devConfig

    override func setUp() async throws {
        try StorageManager.shared.deleteFile(file: CoverDropFiles.statusCache)
    }

    func testFirstRunDoesNotCacheIfAPIResponseFailedAndCacheDisabled() async throws {
        let urlSession = mockApiResponseFailure()

        let results = try await StatusRepository(config: config, urlSession: urlSession)
            .downloadAndUpdateAllCaches(cacheEnabled: false)

        XCTAssertTrue(results?.status == .unavailable)
    }

    func testFirstRunWithOutsideCacheWindowApiResponseLoadsDeadDropsAndCacheDisabled() async throws {
        let urlSession = mockApiResponse()

        // note we are outside the cache window
        let results = try await StatusRepository(
            now: Date(timeIntervalSinceNow: 60 * 60 * 48),
            config: config,
            urlSession: urlSession
        ).downloadAndUpdateAllCaches(cacheEnabled: false)
        XCTAssertTrue(results?.status == .available)
    }

    func testFirstRunReturnsDefaultStatusIfAPIResponseFails() async throws {
        let urlSession = mockApiResponseFailure()

        let results = try await StatusRepository(config: config, urlSession: urlSession)
            .downloadAndUpdateAllCaches()

        XCTAssertTrue(results?.status == .unavailable)
    }

    func testFirstRunWithOutsideCacheWindowApiResponseLoadsDeadDrops() async throws {
        let urlSession = mockApiResponse()

        // note we are outside the cache window
        let results = try await StatusRepository(
            now: Date(timeIntervalSinceNow: 60 * 60 * 48),
            config: config,
            urlSession: urlSession
        ).downloadAndUpdateAllCaches()

        XCTAssertTrue(results?.status == .available)
    }

    func testFirstRunInsideCacheWindowWithApiResponseLoadsStatusAndCachesResponse() async throws {
        let urlSession = mockApiResponse()
        let repo = StatusRepository(
            now: Date(timeIntervalSinceNow: 60 * 50),
            config: config,
            urlSession: urlSession
        )

        // note we are intside the cache window
        let results = try await repo.downloadAndUpdateAllCaches()

        let cache = try await repo.localRepository.load()
        XCTAssertEqual(results, cache)
    }

    func testFutureRunOutsideCacheWindowWithApiResponseLoadsStatusButNotUpdateId() async throws {
        let urlSession = mockApiResponse()
        let repo = StatusRepository(
            now: Date(timeIntervalSinceNow: 60 * 60 * 2),
            config: config,
            urlSession: urlSession
        )

        // note we are outside the cache window
        let results = try await repo.downloadAndUpdateAllCaches()
        let cache = try await repo.localRepository.load()
        XCTAssertEqual(results, cache)
    }

    func testFutureRunInsideCacheWindowWithFailedApiResponseReturnsDefaults() async throws {
        let urlSession = mockApiResponseFailure()

        // note we are inside the cache window
        let results = try await StatusRepository(
            now: Date(timeIntervalSinceNow: 60 * 50),
            config: config,
            urlSession: urlSession
        ).downloadAndUpdateAllCaches(cacheEnabled: false)

        XCTAssertTrue(results?.status == .unavailable)
    }

    func testCacheInsideCacheWindowUpdatesCache() async throws {
        let urlSession = mockApiResponse()

        let results = try await StatusRepository(
            now: Date(timeIntervalSinceNow: 60 * 90),
            config: config,
            urlSession: urlSession
        ).downloadAndUpdateAllCaches()
        let results2 = try await StatusRepository(
            now: Date(timeIntervalSinceNow: 60 * 90),
            config: config,
            urlSession: urlSession
        ).downloadAndUpdateAllCaches()

        XCTAssertEqual(results, results2)
    }

    /// This overrides the default urlSession in our global Config Object, so that calls to our public keys
    /// endpoint
    /// return the mock data supplied
    func mockApiResponse(mockData: [URL?: MockResponse] = [:]) -> URLSession {
        let suppliedMockData = (mockData.isEmpty) ? MockUrlData.getMockUrlData() : mockData

        let urlSessionConfig = URLSessionConfiguration.ephemeral
        URLProtocolMock.mockURLs = suppliedMockData
        urlSessionConfig.protocolClasses = [URLProtocolMock.self]
        let urlSession = URLSession(configuration: urlSessionConfig)
        return urlSession
    }

    func mockApiResponseFailure() -> URLSession {
        let urlSessionConfig = URLSessionConfiguration.ephemeral
        URLProtocolMock.mockURLs = [:]
        urlSessionConfig.protocolClasses = [URLProtocolMock.self]
        let urlSession = URLSession(configuration: urlSessionConfig)
        return urlSession
    }
}
