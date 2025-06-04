import Combine
import Foundation

// MARK: - Implementation

// swiftlint:disable type_name
struct PublicKeyWebRepository: CacheableWebRepository {
    typealias T = PublicKeysData

    func get(params _: [String: String]? = [:]) async throws -> PublicKeysData {
        let result: PublicKeysData = try await call(endpoint: API.allKeys)
        return result
    }

    let urlSession: URLSession
    let baseUrl: String

    init(config: CoverDropConfig, urlSession: URLSession) {
        self.urlSession = urlSession
        baseUrl = config.apiBaseUrl
    }
}

// MARK: - Endpoints

extension PublicKeyWebRepository {
    enum API {
        case allKeys
    }
}

extension PublicKeyWebRepository.API: APICall {
    var path: String? {
        switch self {
        case .allKeys:
            return "/public-keys"
        }
    }

    var method: HttpMethod {
        switch self {
        case .allKeys:
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
