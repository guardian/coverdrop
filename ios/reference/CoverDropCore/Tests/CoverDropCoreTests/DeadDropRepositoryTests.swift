@testable import CoverDropCore
import Sodium
import XCTest

final class DeadDropRepositoryTests: XCTestCase {
    let config: StaticConfig = .devConfig

    override func setUp() async throws {
        try StorageManager.shared.deleteFile(file: CoverDropFiles.deadDropId)
        try StorageManager.shared.deleteFile(file: CoverDropFiles.deadDropCache)
    }

    func fakeUserFacingDeadDrop(id: Int, createdAt: Date) -> DeadDrop {
        let emptyMessage = Base64EncodedString(bytes: "".asBytes())
        let emptyCert = HexEncodedString(bytes: "".asBytes())
        return DeadDrop(id: id, createdAt: RFC3339DateTimeString(date: createdAt), data: emptyMessage, cert: emptyCert)
    }

    func testFirstRunDoesNotCacheIfAPIResponseFailedAndCacheEnabled() async throws {
        let urlSession = mockApiResponseFailure()

        let results = try await DeadDropRepository(config: config, urlSession: urlSession)
            .downloadAndUpdateAllCaches(cacheEnabled: true)

        XCTAssertTrue(results!.deadDrops.isEmpty)
    }

    func testFirstRunDoesNotCacheIfAPIResponseFailedAndCacheDisabled() async throws {
        let urlSession = mockApiResponseFailure()

        let results = try await DeadDropRepository(config: config, urlSession: urlSession)
            .downloadAndUpdateAllCaches(cacheEnabled: false)

        XCTAssertTrue(results!.deadDrops.isEmpty)
    }

    func testFirstRunWithOutsideCacheWindowApiResponseLoadsDeadDropsAndCacheDisabled() async throws {
        let urlSession = mockApiResponse()

        // note we are outside the cache window
        let results = try await DeadDropRepository(
            now: Date(timeIntervalSinceNow: 60 * 60 * 48),
            config: config,
            urlSession: urlSession
        ).downloadAndUpdateAllCaches(cacheEnabled: false)

        guard let actualResults = results else {
            XCTFail("Failed to get results")
            return
        }

        XCTAssertFalse(actualResults.deadDrops.isEmpty)
    }

    func testFirstRunWithOutsideCacheWindowApiResponseLoadsDeadDrops() async throws {
        let urlSession = mockApiResponse()

        // note we are outside the cache window
        let results = try await DeadDropRepository(
            now: Date(timeIntervalSinceNow: 60 * 60 * 48),
            config: config,
            urlSession: urlSession
        ).downloadAndUpdateAllCaches()

        if let validResults = results {
            XCTAssertFalse(validResults.deadDrops.isEmpty)
        } else {
            XCTFail("Failed to get result")
        }
    }

    func testFirstRunInsideCacheWindowWithApiResponseLoadsDeadDropsAndCachesResponse() async throws {
        let urlSession = mockApiResponse()
        let repo = DeadDropRepository(
            now: Date(timeIntervalSinceNow: 60 * 50),
            config: config,
            urlSession: urlSession
        )

        // note we are intside the cache window
        let results = try await repo.downloadAndUpdateAllCaches()
        let cache = try await repo.localRepository.load()
        XCTAssertEqual(results?.deadDrops, cache.deadDrops)
    }

    func testFutureRunOutsideCacheWindowWithApiResponseLoadsDeadDropsButNotUpdateId() async throws {
        let urlSession = mockApiResponse()
        let repo = DeadDropRepository(
            now: Date(timeIntervalSinceNow: 60 * 60 * 2),
            config: config,
            urlSession: urlSession
        )

        // note we are outside the cache window
        let results = try await repo.downloadAndUpdateAllCaches()

        let cache = try await repo.localRepository.load()
        XCTAssertEqual(results?.deadDrops, cache.deadDrops)
    }

    func testFutureRunInsideCacheWindowWithFailedApiResponse() async throws {
        let urlSession = mockApiResponseFailure()

        // note we are inside the cache window
        let results = try await DeadDropRepository(
            now: Date(timeIntervalSinceNow: 60 * 50),
            config: config,
            urlSession: urlSession
        ).downloadAndUpdateAllCaches(cacheEnabled: false)

        guard let actualResults = results else {
            XCTFail("Failed to get results")
            return
        }

        XCTAssertTrue(actualResults.deadDrops.isEmpty)
    }

    func testCacheInsideCacheWindowUpdatesCache() async throws {
        let urlSession = mockApiResponse()

        let results = try await DeadDropRepository(
            now: Date(timeIntervalSinceNow: 60 * 90),
            config: config,
            urlSession: urlSession
        ).downloadAndUpdateAllCaches()
        let results2 = try await DeadDropRepository(
            now: Date(timeIntervalSinceNow: 60 * 90),
            config: config,
            urlSession: urlSession
        ).downloadAndUpdateAllCaches()

        XCTAssertEqual(results?.deadDrops, results2?.deadDrops)
    }

    func testCacheInsideCacheWindowReturnsCachedResults() async throws {
        let urlSession = mockApiResponse()

        guard let deadDropDate = try? PublicKeysHelper.readLocalGeneratedAtFile() else {
            XCTFail("Failed to get deadDrop loaded date")
            return
        }
        // both requests are outside the cache window, so we should be populating the cache
        // on the first request, and reading on the second request
        let deadDropApril01 = fakeUserFacingDeadDrop(id: 10, createdAt: deadDropDate)
        let deadDropApril05 = fakeUserFacingDeadDrop(id: 18, createdAt: deadDropDate)
        let deadDropApril06 = fakeUserFacingDeadDrop(id: 19, createdAt: deadDropDate)

        let cached = DeadDropData(deadDrops: [deadDropApril01, deadDropApril05, deadDropApril06])

        let repo = DeadDropRepository(config: config, urlSession: urlSession)
        try await repo.localRepository.save(data: cached)

        let results = try await DeadDropRepository(
            now: Date(timeIntervalSinceNow: 60 * 90),
            config: config,
            urlSession: urlSession
        ).downloadAndUpdateAllCaches()
        XCTAssertEqual(results?.deadDrops.count, 4)

        let results2 = try await DeadDropRepository(
            now: Date(timeIntervalSinceNow: 60 * 90),
            config: config,
            urlSession: urlSession
        ).downloadAndUpdateAllCaches()
        XCTAssertEqual(results2?.deadDrops.count, 4)
    }

    func testCacheFileDoesNotGetTooLarge() async throws {
        // fill the cache file with 1000 deadDrops
        let maxDeadDropId = 1000
        let twoWeeksInSeconds = 60 * 60 * 24 * 14
        // add a dead drop every 3 mins over 14 days but 2 weeks in the past
        let threeMinsInSeconds = TimeInterval(3 * 60)

        let cachedDeadDrops: [DeadDrop] = generateBulkDeadDrops(
            maxDeadDropId: maxDeadDropId,
            idOffset: 0,
            timeSpan: threeMinsInSeconds,
            timeOffset: TimeInterval(twoWeeksInSeconds)
        )

        // store the dead drops in the cache
        let cached = DeadDropData(deadDrops: cachedDeadDrops)
        let repo = DeadDropRepository(config: config, urlSession: mockApiResponse())
        try await repo.localRepository.save(data: cached)

        // validate the dead drops were saved correctly
        let cachedDeadDropFromFile = try await repo.localRepository.load()
        XCTAssertEqual(cachedDeadDropFromFile.deadDrops.count, 1001)

        // generate more dead drops to be returned as part of the dead drop api mock response
        let apiResponseDeadDropIdOffset = 2000
        let apiResponseDeadDrops: [DeadDrop] = generateBulkDeadDrops(
            maxDeadDropId: maxDeadDropId,
            idOffset: apiResponseDeadDropIdOffset,
            timeSpan: threeMinsInSeconds,
            timeOffset: TimeInterval(0)
        )
        guard let deadDropJsonResponse = try? JSONEncoder().encode(DeadDropData(deadDrops: apiResponseDeadDrops)) else {
            XCTFail("Unable to create mock json dead drop response")
            return
        }

        // This sets up the mock api response
        // In the DeadDropWebRepository we use the highest dead drop Id from the cache as the `ids_greater_than`
        // So this sets up a mock response for that specific request
        let baseUrl = DevConfig().apiBaseUrl
        guard let deadDropAPIPath = DeadDropWebRepository.API
            .allDeadDrops(params: ["ids_greater_than": String(maxDeadDropId)]).path else {
            XCTFail("Could not get path")
            return
        }

        let deadDropDataResponse = Data(deadDropJsonResponse)
        let deadDropMockApiResponse = [URL(string: "\(baseUrl + deadDropAPIPath)")!: MockResponse(
            error: nil,
            data: deadDropDataResponse,
            response: HTTPURLResponse(
                url: URL(string: "\(baseUrl + deadDropAPIPath)")!,
                statusCode: 200,
                httpVersion: nil,
                headerFields: nil
            )!
        )]

        let urlSession = mockApiResponse(mockData: deadDropMockApiResponse)

        // check the file size stays consistant, we do this by checking that
        // the count of dead drops stays at 1001, because all the previous dead drops will have been removed
        // as they are 2 weeks older that the most recent loaded dead drop
        // We check that the highest dead drop id is 3000, which means the files in cache also have the most recent from
        // the api response.

        let results = try await DeadDropRepository(
            now: Date(timeIntervalSinceNow: 60 * 90),
            config: config,
            urlSession: urlSession
        ).downloadAndUpdateAllCaches()
        XCTAssertEqual(results?.deadDrops.count, 1001)
        let highestStoredDeadDrop = results?.deadDrops.max(by: { $0.id < $1.id })?.id
        XCTAssertEqual(highestStoredDeadDrop, 3000)
    }

    func generateBulkDeadDrops(
        maxDeadDropId: Int,
        idOffset: Int,
        timeSpan: TimeInterval,
        timeOffset: TimeInterval
    ) -> [DeadDrop] {
        var cachedDeadDrops: [DeadDrop] = []
        for deadDropId in Array(0 ... maxDeadDropId) {
            let deadDropDate = Date(timeIntervalSinceNow: -((Double(deadDropId) * timeSpan) + timeOffset))

            let deadDrop = fakeUserFacingDeadDrop(id: deadDropId + idOffset, createdAt: deadDropDate)
            cachedDeadDrops.append(deadDrop)
        }
        return cachedDeadDrops
    }

    /// This overrides the default UrlSessionConfig in our global Config Object, so that calls to our public keys
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

    func testDeadDrops_whenDownloadedWithNonEmptyStorage_thenMergedTrimmedAndAvailableAsMostRecent() async throws {
        let deadDropApril01 = fakeUserFacingDeadDrop(
            id: 10,
            createdAt: DateFormats.validateDate(date: "2023-04-01T00:00:00Z") ?? DateFunction.currentTime()
        )
        let deadDropApril05 = fakeUserFacingDeadDrop(
            id: 20,
            createdAt: DateFormats.validateDate(date: "2023-04-05T00:00:00Z") ?? DateFunction.currentTime()
        )
        let deadDropApril06 = fakeUserFacingDeadDrop(
            id: 21,
            createdAt: DateFormats.validateDate(date: "2023-04-06T00:00:00Z") ?? DateFunction.currentTime()
        )
        let deadDropApril10 = fakeUserFacingDeadDrop(
            id: 40,
            createdAt: DateFormats.validateDate(date: "2023-04-10T00:00:00Z") ?? DateFunction.currentTime()
        )
        let deadDropApril11 = fakeUserFacingDeadDrop(
            id: 50,
            createdAt: DateFormats.validateDate(date: "2023-04-11T00:00:00Z") ?? DateFunction.currentTime()
        )
        let deadDropApril20 = fakeUserFacingDeadDrop(
            id: 80,
            createdAt: DateFormats.validateDate(date: "2023-04-20T00:00:00Z") ?? DateFunction.currentTime()
        )
        let deadDropJune01 = fakeUserFacingDeadDrop(
            id: 200,
            createdAt: DateFormats.validateDate(date: "2023-06-01T00:00:00Z") ?? DateFunction.currentTime()
        )
        let deadDropJune07 = fakeUserFacingDeadDrop(
            id: 201,
            createdAt: DateFormats.validateDate(date: "2023-06-01T00:00:00Z") ?? DateFunction.currentTime()
        )

        // Start with an empty storage
        var existingDeadDrops = DeadDropData(deadDrops: [])

        // Add dead drops on April 10 that range from April 1 to April 10
        let newDeadDropsApril10 = DeadDropData(deadDrops:
            [deadDropApril01, deadDropApril05, deadDropApril06, deadDropApril10])

        // After merging and trimming we expect that we only have dead drops that range from
        // April 1 to April 10. I.e., all of them
        existingDeadDrops = DeadDropRepository.mergeAndTrim(
            existingDeadDrops: existingDeadDrops,
            newDeadDrops: newDeadDropsApril10
        )
        XCTAssertTrue(existingDeadDrops.deadDrops.containsExactly([
            deadDropApril01,
            deadDropApril05,
            deadDropApril06,
            deadDropApril10
        ]))

        // Add dead drops on April 20 that range from April 11 to April 20
        let newDeadDropsApril20 = DeadDropData(deadDrops:
            [deadDropApril11, deadDropApril20])

        // After merging and trimming we expect that we only have dead drops that range from
        // April 6 to April 20 (i.e. deadDropCacheTTL).
        existingDeadDrops = DeadDropRepository.mergeAndTrim(
            existingDeadDrops: existingDeadDrops,
            newDeadDrops: newDeadDropsApril20
        )
        XCTAssertTrue(existingDeadDrops.deadDrops.containsExactly(
            [deadDropApril06, // just barely in by 1 second because the cut-off-date is inclusive
             deadDropApril10,
             deadDropApril11,
             deadDropApril20]
        ))

        // Add dead drops on June 7 that range from June 1 to June 7
        let newDeadDropsJune07 = DeadDropData(deadDrops:
            [deadDropJune01, deadDropJune07])

        // After merging and trimming we expect that we only have dead drops that range from
        // June 1 to June 7.
        existingDeadDrops = DeadDropRepository.mergeAndTrim(
            existingDeadDrops: existingDeadDrops,
            newDeadDrops: newDeadDropsJune07
        )
        XCTAssertTrue(existingDeadDrops.deadDrops.containsExactly(
            [deadDropJune01,
             deadDropJune07]
        ))
    }
}
