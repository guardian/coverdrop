import Combine
import Foundation

// MARK: - Implementation

// swiftlint:disable type_name

struct DeadDropWebRepository: CacheableWebRepository {
    typealias T = DeadDropData

    let urlSession: URLSession
    let baseUrl: String

    func get(params: [String: String]?) async throws -> DeadDropData {
        return try await call(endpoint: API.allDeadDrops(params: params))
    }
}

// MARK: - Endpoints

extension DeadDropWebRepository {
    enum API {
        case allDeadDrops(params: [String: String]?)
    }
}

extension DeadDropWebRepository.API: APICall {
    var path: String? {
        switch self {
        case let .allDeadDrops(params: params):
            guard let params,
                  let createdAfter = params["created_after"] else {
                return nil
            }
            return "/journalist-to-user/dead-drops?created_after=\(createdAfter)"
        }
    }

    var method: HttpMethod {
        switch self {
        case .allDeadDrops:
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

// swiftlint:enable type_name
