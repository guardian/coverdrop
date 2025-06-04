import Foundation

public enum UserToJournalistMessagingError: Error {
    case unableToBase64Encode
    case failedToPeekMessage
    case failedToSendMessage
    case failedToDequeue
    case failedToGetConfig
}

// MARK: - Implementation

public struct UserToJournalistMessageWebRepository: WebRepository {
    let urlSession: URLSession
    let baseUrl: String

    public init(urlSession: URLSession, baseUrl: String) {
        self.baseUrl = baseUrl
        self.urlSession = urlSession
    }

    public func sendMessage(jsonData: Data) async throws -> HTTPURLResponse {
        return try await post(endpoint: API.sendMessage, body: jsonData)
    }
}

// MARK: - Endpoints

extension UserToJournalistMessageWebRepository {
    enum API {
        case sendMessage
    }
}

extension UserToJournalistMessageWebRepository.API: APICall {
    var path: String? {
        switch self {
        case .sendMessage:
            return "/user/messages"
        }
    }

    var method: HttpMethod {
        switch self {
        case .sendMessage:
            return .POST
        }
    }

    var headers: [String: String]? {
        return ["Accept": "application/json", "Content-Type": "application/json"]
    }
}
