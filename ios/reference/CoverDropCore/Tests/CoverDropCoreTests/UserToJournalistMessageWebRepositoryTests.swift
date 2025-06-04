@testable import CoverDropCore
import Sodium
import XCTest

final class UserToJournalistMessageRepositoryTests: XCTestCase {
    func testSendMessageError() async throws {
        let urlSession = mockApiResponseFailure()

        let numRecipients = 1
        let bytesLength = (numRecipients * Sodium().secretBox.KeyBytes + Sodium().box.SealBytes) + Sodium().secretBox
            .MacBytes

        let bytes = Sodium().randomBytes.buf(length: bytesLength)!

        let message = MultiAnonymousBox<UserToCoverNodeMessageData>(bytes: bytes)
        guard let data = message.asBytes().base64Encode() else {
            XCTFail("unable to encode data")
            return
        }
        let jsonData: Data = try JSONEncoder().encode(data)
        do {
            _ = try await UserToJournalistMessageWebRepository(
                urlSession: urlSession,
                baseUrl: StaticConfig.devConfig.messageBaseUrl
            ).sendMessage(jsonData: jsonData)
            XCTFail("API error should have failed")
        } catch {}
    }

    func testSendMessageSuccess() async throws {
        let urlSession = mockApiResponse()

        let numRecipients = 1
        let bytesLength = (numRecipients * Sodium().secretBox.KeyBytes + Sodium().box.SealBytes) + Sodium().secretBox
            .MacBytes

        let bytes = Sodium().randomBytes.buf(length: bytesLength)!

        let message = MultiAnonymousBox<UserToCoverNodeMessageData>(bytes: bytes)
        guard let data = message.asBytes().base64Encode() else {
            XCTFail("unable to encode data")
            return
        }
        let jsonData: Data = try JSONEncoder().encode(data)
        do {
            _ = try await UserToJournalistMessageWebRepository(
                urlSession: urlSession,
                baseUrl: StaticConfig.devConfig.messageBaseUrl
            ).sendMessage(jsonData: jsonData)
        } catch {
            XCTFail("API error should have failed")
        }
    }

    /// This overrides the default UrlSessionConfig in our global Config Object, so that calls to our public keys
    /// endpoint
    /// return the mock data supplied
    func mockApiResponse() -> URLSession {
        let urlSessionConfig = URLSessionConfiguration.ephemeral
        URLProtocolMock.mockURLs = MockUrlData.getMockUrlData()
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
