@testable import CoverDropCore
import Sodium
import XCTest

final class DeadDropRepositoryTests: XCTestCase {
    let config: StaticConfig = .devConfig
    let testKeys = PublicKeysHelper.shared.testKeys

    override func setUp() async throws {
        try StorageManager.shared.deleteFile(file: CoverDropFiles.deadDropId)
        try StorageManager.shared.deleteFile(file: CoverDropFiles.deadDropCache)
    }

    /// By default the content (data and signature) is derived from the `id`, so dead drops with different ids are
    /// different dead drops. Pass the same `contentSeed` to create copies of the same signed dead drop under
    /// different ids.
    func fakeUserFacingDeadDrop(id: Int, createdAt: Date, contentSeed: Int? = nil) -> DeadDrop {
        let seed = contentSeed ?? id
        let message = Base64EncodedString(bytes: "data-\(seed)".asBytes())
        let signature = HexEncodedString(bytes: "signature-\(seed)".asBytes())
        return DeadDrop(
            id: id,
            createdAt: RFC3339DateTimeString(date: createdAt),
            data: message,
            signature: signature
        )
    }

    func testFirstRunDoesNotCacheIfAPIResponseFailedAndCacheEnabled() async throws {
        let urlSession = mockApiResponseFailure()

        let results = try await DeadDropRepository(config: config, urlSession: urlSession)
            .downloadAndUpdateAllCaches(cacheEnabled: true, verifiedKeys: testKeys)

        XCTAssertTrue(try XCTUnwrap(results?.deadDrops.isEmpty))
    }

    func testFirstRunDoesNotCacheIfAPIResponseFailedAndCacheDisabled() async throws {
        let urlSession = mockApiResponseFailure()

        let results = try await DeadDropRepository(config: config, urlSession: urlSession)
            .downloadAndUpdateAllCaches(cacheEnabled: false, verifiedKeys: testKeys)

        XCTAssertTrue(try XCTUnwrap(results?.deadDrops.isEmpty))
    }

    func testFirstRunWithOutsideCacheWindowApiResponseLoadsDeadDropsAndCacheDisabled() async throws {
        let urlSession = mockApiResponse()

        // note we are outside the cache window
        let results = try await DeadDropRepository(
            now: Date(timeIntervalSinceNow: 60 * 60 * 48),
            config: config,
            urlSession: urlSession
        ).downloadAndUpdateAllCaches(cacheEnabled: false, verifiedKeys: testKeys)

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
        ).downloadAndUpdateAllCaches(verifiedKeys: testKeys)

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
        let results = try await repo.downloadAndUpdateAllCaches(verifiedKeys: testKeys)
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
        let results = try await repo.downloadAndUpdateAllCaches(verifiedKeys: testKeys)

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
        ).downloadAndUpdateAllCaches(cacheEnabled: false, verifiedKeys: testKeys)

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
        ).downloadAndUpdateAllCaches(verifiedKeys: testKeys)
        let results2 = try await DeadDropRepository(
            now: Date(timeIntervalSinceNow: 60 * 90),
            config: config,
            urlSession: urlSession
        ).downloadAndUpdateAllCaches(verifiedKeys: testKeys)

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
        ).downloadAndUpdateAllCaches(verifiedKeys: testKeys)
        XCTAssertEqual(results?.deadDrops.count, 4)

        let results2 = try await DeadDropRepository(
            now: Date(timeIntervalSinceNow: 60 * 90),
            config: config,
            urlSession: urlSession
        ).downloadAndUpdateAllCaches(verifiedKeys: testKeys)
        XCTAssertEqual(results2?.deadDrops.count, 4)
    }

    func testCacheFileDoesNotGetTooLarge() async throws {
        let verifiedKeys = try IntegrationTestScenarioContext(scenario: .messaging)
            .loadKeysVerified(step: "003_journalist_replied_and_processed")
        // These verify, so the newest one is the cursor and the reference for trimming
        let genuineDeadDrops = try DeadDropDataHelper.shared.readLocalDataFile().deadDrops
        let newestGenuine = try XCTUnwrap(genuineDeadDrops.map(\.createdAt).max())

        let ttl = TimeInterval(Constants.clientDeadDropCacheTtlSeconds)
        let threeMinsInSeconds = TimeInterval(3 * 60)
        let fakeCount = 1000

        // fakes that do not verify, one every 3 mins, all just outside the TTL (plus a margin, because timestamps are
        // truncated to whole seconds) relative to the newest genuine dead drop
        let staleDeadDrops = (0 ... fakeCount).map { index in
            fakeUserFacingDeadDrop(
                id: index,
                createdAt: newestGenuine.date.addingTimeInterval(-(ttl + 10 + Double(index) * threeMinsInSeconds))
            )
        }

        let repo = DeadDropRepository(config: config, urlSession: mockApiResponse())
        try await repo.localRepository.save(data: DeadDropData(deadDrops: genuineDeadDrops + staleDeadDrops))

        // validate the dead drops were saved correctly
        let cachedDeadDropFromFile = try await repo.localRepository.load()
        XCTAssertEqual(cachedDeadDropFromFile.deadDrops.count, genuineDeadDrops.count + staleDeadDrops.count)

        // the api returns more dead drops that are newer than the cursor
        let apiResponseDeadDropIdOffset = 2000
        let apiResponseDeadDrops = (0 ... fakeCount).map { index in
            fakeUserFacingDeadDrop(
                id: apiResponseDeadDropIdOffset + index,
                createdAt: newestGenuine.date.addingTimeInterval(1 + Double(index) * threeMinsInSeconds)
            )
        }
        let deadDropJsonResponse = try JSONEncoder().encode(DeadDropData(deadDrops: apiResponseDeadDrops))

        // This sets up the mock api response for the `created_after` cursor of the newest genuine dead drop
        let baseUrl = DevConfig().apiBaseUrl
        let deadDropAPIPath = try XCTUnwrap(DeadDropWebRepository.API
            .allDeadDrops(params: ["created_after": newestGenuine.rfc3339SecondString]).path)
        let mockUrl = try XCTUnwrap(URL(string: "\(baseUrl + deadDropAPIPath)"))
        let deadDropMockApiResponse = try [mockUrl: MockResponse(
            error: nil,
            data: deadDropJsonResponse,
            response: XCTUnwrap(HTTPURLResponse(url: mockUrl, statusCode: 200, httpVersion: nil, headerFields: nil))
        )]

        let results = try await DeadDropRepository(
            now: Date(timeIntervalSinceNow: 60 * 90),
            config: config,
            urlSession: mockApiResponse(mockData: deadDropMockApiResponse)
        ).downloadAndUpdateAllCaches(verifiedKeys: verifiedKeys)

        // the stale dead drops are evicted, so the cache holds the genuine and the newly downloaded dead drops
        let resultingDeadDrops = try XCTUnwrap(results?.deadDrops)
        XCTAssertEqual(resultingDeadDrops.count, genuineDeadDrops.count + apiResponseDeadDrops.count)
        XCTAssertTrue(genuineDeadDrops.allSatisfy { resultingDeadDrops.contains($0) })
        XCTAssertTrue(staleDeadDrops.allSatisfy { !resultingDeadDrops.contains($0) })
        XCTAssertEqual(resultingDeadDrops.map(\.id).max(), apiResponseDeadDropIdOffset + fakeCount)
    }

    /// This overrides the default UrlSessionConfig in our global Config Object, so that calls to our public keys
    /// endpoint
    /// return the mock data supplied
    func mockApiResponse(mockData: [URL?: MockResponse] = [:]) -> URLSession {
        let suppliedMockData = (mockData.isEmpty) ? MockUrlData.getMockUrlData() : mockData

        let urlSessionConfig = URLSessionConfiguration.ephemeral
        URLProtocolMock.mockURLs = suppliedMockData
        urlSessionConfig.protocolClasses = [URLProtocolMock.self]
        return URLSession(configuration: urlSessionConfig)
    }

    func mockApiResponseFailure() -> URLSession {
        let urlSessionConfig = URLSessionConfiguration.ephemeral
        URLProtocolMock.mockURLs = [:]
        urlSessionConfig.protocolClasses = [URLProtocolMock.self]
        return URLSession(configuration: urlSessionConfig)
    }

    func testDeadDrops_whenDownloadedWithNonEmptyStorage_thenMergedTrimmedAndAvailableAsMostRecent() {
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
            newDeadDrops: newDeadDropsApril10,
            trimReference: deadDropApril10.createdAt
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
            newDeadDrops: newDeadDropsApril20,
            trimReference: deadDropApril20.createdAt
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
            newDeadDrops: newDeadDropsJune07,
            trimReference: deadDropJune07.createdAt
        )
        XCTAssertTrue(existingDeadDrops.deadDrops.containsExactly(
            [deadDropJune01,
             deadDropJune07]
        ))
    }

    func testDiscardDeadDropsAtOrBeforeCursor_discardsDeadDropsNotAfterCursor() {
        let cursor = RFC3339DateTimeString(
            date: DateFormats.validateDate(date: "2023-04-10T00:00:00Z") ?? DateFunction.currentTime()
        )
        let deadDropBeforeCursor = fakeUserFacingDeadDrop(
            id: 1,
            createdAt: DateFormats.validateDate(date: "2023-04-09T00:00:00Z") ?? DateFunction.currentTime()
        )
        let deadDropAtCursor = fakeUserFacingDeadDrop(id: 2, createdAt: cursor.date)
        let deadDropAfterCursor = fakeUserFacingDeadDrop(
            id: 3,
            createdAt: DateFormats.validateDate(date: "2023-04-11T00:00:00Z") ?? DateFunction.currentTime()
        )

        let filtered = DeadDropRepository.discardDeadDropsAtOrBeforeCursor(
            deadDrops: DeadDropData(deadDrops: [deadDropBeforeCursor, deadDropAtCursor, deadDropAfterCursor]),
            cursor: cursor
        )

        XCTAssertTrue(filtered.deadDrops.containsExactly([deadDropAfterCursor]))
    }

    // MARK: - Second precision and deduplication by signed content

    func testNormalizeAndDeduplicate_truncatesCreatedAtToWholeSeconds() throws {
        let createdAt = try XCTUnwrap(DateFormats.validateDate(date: "2023-04-10T12:34:56.789000Z"))
        let deadDropData = DeadDropData(deadDrops: [fakeUserFacingDeadDrop(id: 1, createdAt: createdAt)])

        let normalized = DeadDropRepository.normalizeAndDeduplicate(deadDropData)

        let normalizedCreatedAt = try XCTUnwrap(normalized.deadDrops.first?.createdAt)
        XCTAssertEqual(normalizedCreatedAt.date.timeIntervalSince1970, createdAt.timeIntervalSince1970.rounded(.down))
        XCTAssertEqual(normalizedCreatedAt.rfc3339SecondString, "2023-04-10T12:34:56Z")
    }

    func testDeadDropIdentity_ignoresIdAndSubSecondTimestamp() throws {
        let createdAt = try XCTUnwrap(DateFormats.validateDate(date: "2023-04-10T00:00:00Z"))
        let original = fakeUserFacingDeadDrop(id: 1, createdAt: createdAt, contentSeed: 7)

        let differentId = fakeUserFacingDeadDrop(id: 2, createdAt: createdAt, contentSeed: 7)
        let differentSubSecond = fakeUserFacingDeadDrop(
            id: 1,
            createdAt: createdAt.addingTimeInterval(0.389),
            contentSeed: 7
        )

        XCTAssertEqual(original, differentId)
        XCTAssertEqual(original.hashValue, differentId.hashValue)
        XCTAssertEqual(original, differentSubSecond)
        XCTAssertEqual(original.hashValue, differentSubSecond.hashValue)
    }

    func testDeadDropIdentity_distinguishesSignedContent() throws {
        let createdAt = try XCTUnwrap(DateFormats.validateDate(date: "2023-04-10T00:00:00Z"))
        let original = fakeUserFacingDeadDrop(id: 1, createdAt: createdAt, contentSeed: 7)

        // different whole second
        let nextSecond = fakeUserFacingDeadDrop(id: 1, createdAt: createdAt.addingTimeInterval(1), contentSeed: 7)
        XCTAssertNotEqual(original, nextSecond)
        // different data and signature
        XCTAssertNotEqual(original, fakeUserFacingDeadDrop(id: 1, createdAt: createdAt, contentSeed: 8))

        // same signature but tampered data: this must not be mistaken for the genuine dead drop, otherwise serving it
        // first would block the genuine dead drop
        var tamperedData = original
        tamperedData.data = Base64EncodedString(bytes: "tampered".asBytes())
        XCTAssertNotEqual(original, tamperedData)
    }

    func testMergeAndTrim_mergesCopiesOfSameSignedDeadDropWithDifferentIds() throws {
        let createdAt = try XCTUnwrap(DateFormats.validateDate(date: "2023-04-10T00:00:00Z"))

        // identical data, signature and `createdAt`; only the unsigned `id` differs
        let replayedCopies = (0 ..< 1000).map { index in
            fakeUserFacingDeadDrop(id: 1000 + index, createdAt: createdAt, contentSeed: 0)
        }

        let merged = DeadDropRepository.mergeAndTrim(
            existingDeadDrops: DeadDropData(deadDrops: []),
            newDeadDrops: DeadDropData(deadDrops: replayedCopies),
            trimReference: RFC3339DateTimeString(date: createdAt)
        )

        XCTAssertEqual(merged.deadDrops.count, 1)
    }

    func testMergeAndTrim_mergesCopiesOfSameSignedDeadDropWithDifferentSubSecondTimestamps() throws {
        let signedSecond = try XCTUnwrap(DateFormats.validateDate(date: "2023-04-10T00:00:00Z"))

        // identical id, data and signature; only the unsigned sub-second part of `createdAt` differs
        let replayedCopies = (0 ..< 1000).map { millisecond in
            fakeUserFacingDeadDrop(
                id: 1,
                createdAt: signedSecond.addingTimeInterval(TimeInterval(millisecond) / 1000),
                contentSeed: 0
            )
        }

        let merged = DeadDropRepository.mergeAndTrim(
            existingDeadDrops: DeadDropData(deadDrops: []),
            newDeadDrops: DeadDropData(deadDrops: replayedCopies),
            trimReference: RFC3339DateTimeString(date: signedSecond)
        )

        XCTAssertEqual(merged.deadDrops.count, 1)
        XCTAssertEqual(merged.deadDrops.first?.createdAt.rfc3339SecondString, "2023-04-10T00:00:00Z")
    }

    func testMergeAndTrim_normalizesLegacyCachedDeadDropsWithSubSecondPrecision() throws {
        let signedSecond = try XCTUnwrap(DateFormats.validateDate(date: "2023-04-10T12:34:56Z"))
        let legacyCached = DeadDropData(deadDrops: [
            fakeUserFacingDeadDrop(id: 1, createdAt: signedSecond.addingTimeInterval(0.389), contentSeed: 0)
        ])
        let sameDeadDropServedAgain = DeadDropData(deadDrops: [
            fakeUserFacingDeadDrop(id: 2, createdAt: signedSecond, contentSeed: 0)
        ])

        let merged = DeadDropRepository.mergeAndTrim(
            existingDeadDrops: legacyCached,
            newDeadDrops: sameDeadDropServedAgain,
            trimReference: RFC3339DateTimeString(date: signedSecond)
        )

        XCTAssertEqual(merged.deadDrops.count, 1)
        XCTAssertEqual(merged.deadDrops.first?.createdAt.rfc3339SecondString, "2023-04-10T12:34:56Z")
    }

    func testCursorFilterAndMergeAndTrim_acceptReplayedCopiesAfterCursorOnlyOnce() throws {
        let cursorDate = try XCTUnwrap(DateFormats.validateDate(date: "2023-04-10T00:00:00Z"))
        let cursor = RFC3339DateTimeString(date: cursorDate)
        let existing = DeadDropData(deadDrops: [fakeUserFacingDeadDrop(id: 1, createdAt: cursorDate)])

        // A malicious API serves one genuinely signed new dead drop many times under different ids
        let newCreatedAt = cursorDate.addingTimeInterval(1)
        let replayedCopies = (0 ..< 1000).map { index in
            fakeUserFacingDeadDrop(id: 2 + index, createdAt: newCreatedAt, contentSeed: 2)
        }

        // This is the sequence used by `getFromApiAndCache`
        let filtered = DeadDropRepository.discardDeadDropsAtOrBeforeCursor(
            deadDrops: DeadDropRepository.normalizeAndDeduplicate(DeadDropData(deadDrops: replayedCopies)),
            cursor: cursor
        )
        let merged = DeadDropRepository.mergeAndTrim(
            existingDeadDrops: existing,
            newDeadDrops: filtered,
            trimReference: cursor
        )

        XCTAssertEqual(filtered.deadDrops.count, 1)
        // the existing dead drop and one copy of the new dead drop
        XCTAssertEqual(merged.deadDrops.count, 2)
    }

    func testCursorFilter_discardsCursorDeadDropServedAgainWithSubSecondPrecision() throws {
        // The API returns dead drops with `created_at > cursor` and its `created_at` has sub-second precision.
        // Because the cursor is truncated to whole seconds, the dead drop that the cursor was taken from is served
        // again. After truncating it is equal to the cursor, so it is discarded.
        let cursorDate = try XCTUnwrap(DateFormats.validateDate(date: "2023-04-10T00:00:00Z"))
        let servedAgain = fakeUserFacingDeadDrop(id: 1, createdAt: cursorDate.addingTimeInterval(0.389))

        let filtered = DeadDropRepository.discardDeadDropsAtOrBeforeCursor(
            deadDrops: DeadDropRepository.normalizeAndDeduplicate(DeadDropData(deadDrops: [servedAgain])),
            cursor: RFC3339DateTimeString(date: cursorDate)
        )

        XCTAssertTrue(filtered.deadDrops.isEmpty)
    }

    // MARK: - Verification of the cursor

    func testGetFromApiAndCache_withoutVerifiedKeys_doesNotPullAndKeepsCache() async throws {
        let createdAt = try XCTUnwrap(DateFormats.validateDate(date: "2023-04-10T00:00:00Z"))
        let cachedDeadDrop = fakeUserFacingDeadDrop(id: 1, createdAt: createdAt)

        // every request fails, so pulling would replace the cache content with the default response
        let repo = DeadDropRepository(config: config, urlSession: mockApiResponseFailure())
        try await repo.localRepository.save(data: DeadDropData(deadDrops: [cachedDeadDrop]))

        let results = await repo.getFromApiAndCache(verifiedKeys: nil)

        XCTAssertEqual(results?.deadDrops, [cachedDeadDrop])
    }

    func testGetFromApiAndCache_cursorIgnoresCachedDeadDropsThatDoNotVerify() async throws {
        let verifiedKeys = try IntegrationTestScenarioContext(scenario: .messaging)
            .loadKeysVerified(step: "003_journalist_replied_and_processed")
        let genuineDeadDrops = try DeadDropDataHelper.shared.readLocalDataFile().deadDrops

        // A malicious API served a dead drop with a made-up `createdAt`. It is part of what is signed, so the dead drop
        // does not verify.
        var forgedDeadDrop = try XCTUnwrap(genuineDeadDrops.max(by: { $0.createdAt < $1.createdAt }))
        forgedDeadDrop.createdAt = RFC3339DateTimeString(
            date: forgedDeadDrop.createdAt.date.addingTimeInterval(24 * 60 * 60)
        )

        let repo = DeadDropRepository(config: config, urlSession: mockApiResponse())
        try await repo.localRepository.save(data: DeadDropData(deadDrops: genuineDeadDrops + [forgedDeadDrop]))

        let results = await repo.getFromApiAndCache(verifiedKeys: verifiedKeys)

        // The mock API only answers requests that use the `createdAt` of the newest genuine dead drop (or the epoch)
        // as the cursor. If the forged dead drop had moved the cursor, the request would have failed and the result
        // would be empty.
        let resultingDeadDrops = try XCTUnwrap(results?.deadDrops)
        XCTAssertTrue(genuineDeadDrops.allSatisfy { resultingDeadDrops.contains($0) })
    }
}
