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
        return [
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
            // In our mock data, we are using dead-drop id 19 ... just because
            URL(string: "http://localhost:3000/v1/user/dead-drops?ids_greater_than=19")!: MockResponse(
                error: nil,
                data: deadDropData,
                response: HTTPURLResponse(
                    url: URL(string: "http://localhost:3000/v1/user/dead-drops?ids_greater_than=19")!,
                    statusCode: 200,
                    httpVersion: nil,
                    headerFields: nil
                )!
            ),
            // In our mock data, we are using dead-drop id 0 ... just because
            URL(string: "http://localhost:3000/v1/user/dead-drops?ids_greater_than=0")!: MockResponse(
                error: nil,
                data: deadDropData,
                response: HTTPURLResponse(
                    url: URL(string: "http://localhost:3000/v1/user/dead-drops?ids_greater_than=0")!,
                    statusCode: 200,
                    httpVersion: nil,
                    headerFields: nil
                )!
            ),
            // In our mock data, we are using dead-drop id 0 ... just because
            URL(string: "http://localhost:3000/v1/user/dead-drops?ids_greater_than=1")!: MockResponse(
                error: nil,
                data: deadDropData,
                response: HTTPURLResponse(
                    url: URL(string: "http://localhost:3000/v1/user/dead-drops?ids_greater_than=0")!,
                    statusCode: 200,
                    httpVersion: nil,
                    headerFields: nil
                )!
            ),
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
    }

    static func getDeadDrops() -> Data {
        do {
            return try DeadDropDataHelper.shared.readLoadDeadDropJson()
        } catch {
            return Data()
        }
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
