import Combine
import Foundation

// MARK: - Implementation

// swiftlint:disable type_name
struct StatusWebRepository: CacheableWebRepository {
    typealias T = StatusData
    let urlSession: URLSession
    let baseUrl: String

    func get(params _: [String: String]?) async throws -> StatusData {
        let response: StatusData = try await call(endpoint: API.status)
        return response
    }
}

// swiftlint:enable type_name

// MARK: - Endpoints

extension StatusWebRepository {
    enum API {
        case status
    }
}

extension StatusWebRepository.API: APICall {
    var path: String? {
        switch self {
        case .status:
            return "/status"
        }
    }

    var method: HttpMethod {
        switch self {
        case .status:
            return .GET
        }
    }

    var headers: [String: String]? {
        return ["Accept": "application/json"]
    }

    func body() throws -> Data? {
        return nil
    }
}
