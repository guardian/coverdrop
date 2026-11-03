import Foundation

struct MockResponse {
    let error: Error?
    let data: Data
    let response: HTTPURLResponse
}

enum MockUrlData {
    static func getMockUrlData() -> [URL?: MockResponse] {
        var publicKeysData = MockUrlData.getKeys()
        var deadDropData = MockUrlData.getDeadDrops()
        var statusData = MockUrlData.getStatusData()
        #if DEBUG
            if TestingBridge.isEnabled(.mockedDataEmptyKeysData) {
                publicKeysData = Data()
            } else if TestingBridge.isEnabled(.mockedDataMultipleJournalists) {
                publicKeysData = MockUrlData.getMultipleJournalistKeys()
                deadDropData = MockUrlData.getMulitpleJournalistDeadDrops()
            } else if TestingBridge.isEnabled(.mockedDataNoDefaultJournalist) {
                publicKeysData = MockUrlData.getJournalistKeysNoDefaultJournalist()
            }
            if TestingBridge.isEnabled(.mockedDataStatusUnavailable) {
                statusData = MockUrlData.getStatusUnavailableData()
            }
        #endif

        // Subsequent dead drop fetches use the `createdAt` of the highest dead drop in the active fixture as the
        // cursor,
        // so this is derived from the fixture rather than hardcoded, to stay valid if the test vectors are regenerated.
        let epochCreatedAfter = Date(timeIntervalSince1970: 0).rfc3339MicrosecondString
        let subsequentFetchCreatedAfter = MockUrlData.highestCreatedAtCursor(for: deadDropData)
            ?? epochCreatedAfter

        var mockUrls: [URL?: MockResponse] = [
            URL(string: "http://localhost:3000/v1/public-keys")!: MockResponse(
                error: nil,
                data: publicKeysData,
                response: HTTPURLResponse(
                    url: URL(string: "http://localhost:3000/v1/public-keys")!,
                    statusCode: 200,
                    httpVersion: nil,
                    headerFields: nil
                )!
            ),
            // The first ever fetch has no cache, so the client defaults the cursor to the epoch
            MockUrlData.deadDropsUrl(createdAfter: epochCreatedAfter):
                MockUrlData.deadDropsMockResponse(data: deadDropData, createdAfter: epochCreatedAfter),
            // Subsequent fetches use the `createdAt` of the highest dead drop in the active fixture as the cursor
            MockUrlData.deadDropsUrl(createdAfter: subsequentFetchCreatedAfter):
                MockUrlData.deadDropsMockResponse(data: deadDropData, createdAfter: subsequentFetchCreatedAfter),
            URL(string: "http://localhost:7676/user/messages")!: MockResponse(
                error: nil,
                data: Data(),
                response: HTTPURLResponse(
                    url: URL(string: "http://localhost:7676/user/messages")!,
                    statusCode: 200,
                    httpVersion: nil,
                    headerFields: nil
                )!
            ),
            URL(string: "http://localhost:3000/v1/status")!: MockResponse(
                error: nil,
                data: statusData,
                response: HTTPURLResponse(
                    url: URL(string: "http://localhost:3000/v1/status")!,
                    statusCode: 200,
                    httpVersion: nil,
                    headerFields: nil
                )!
            )
        ]

        // Some tests seed the dead drop cache using our test keys' generated-at timestamp, so we also need
        // a mock response for a fetch cursored from that date
        if let keysGeneratedAt = (try? PublicKeysHelper.readLocalGeneratedAtFile()) ?? nil {
            let keysGeneratedAtCreatedAfter = keysGeneratedAt.rfc3339MicrosecondString
            mockUrls[MockUrlData.deadDropsUrl(createdAfter: keysGeneratedAtCreatedAfter)] =
                MockUrlData.deadDropsMockResponse(data: deadDropData, createdAfter: keysGeneratedAtCreatedAfter)
        }

        return mockUrls
    }

    static func deadDropsUrl(createdAfter: String) -> URL {
        URL(string: "http://localhost:3000/v1/journalist-to-user/dead-drops?created_after=\(createdAfter)")!
    }

    static func deadDropsMockResponse(data: Data, createdAfter: String) -> MockResponse {
        let url = MockUrlData.deadDropsUrl(createdAfter: createdAfter)
        return MockResponse(
            error: nil,
            data: data,
            response: HTTPURLResponse(
                url: url,
                statusCode: 200,
                httpVersion: nil,
                headerFields: nil
            )!
        )
    }

    static func getDeadDrops() -> Data {
        do {
            return try DeadDropDataHelper.shared.readLoadDeadDropJson()
        } catch {
            return Data()
        }
    }

    static func highestCreatedAtCursor(for deadDropJson: Data) -> String? {
        guard let deadDropData = try? JSONDecoder().decode(DeadDropData.self, from: deadDropJson),
              let highestCreatedAt = deadDropData.deadDrops.max(by: { $0.createdAt < $1.createdAt })?.createdAt else {
            return nil
        }
        return highestCreatedAt.rfc3339MicrosecondString
    }

    static func getStatusData() -> Data {
        do {
            return try StatusDataHelper.shared.readAvailableStatusJson()
        } catch {
            return Data()
        }
    }

    static func getStatusUnavailableData() -> Data {
        do {
            return try StatusDataHelper.shared.readUnavailableStatusJson()
        } catch {
            return Data()
        }
    }

    static func getMulitpleJournalistDeadDrops() -> Data {
        do {
            return try DeadDropDataHelper.shared.readLoadMultipleJournalistDeadDropJson()
        } catch {
            return Data()
        }
    }

    static func getKeys() -> Data {
        do {
            return try PublicKeysHelper.readLocalKeysJson()
        } catch {
            return Data()
        }
    }

    static func getMultipleJournalistKeys() -> Data {
        do {
            return try PublicKeysHelper.readLocalMultipleMessagingKeysJson()
        } catch {
            return Data()
        }
    }

    static func getJournalistKeysNoDefaultJournalist() -> Data {
        do {
            return try PublicKeysHelper.readLocalMessagingKeysNoDefaultJournalistJson()
        } catch {
            return Data()
        }
    }
}
