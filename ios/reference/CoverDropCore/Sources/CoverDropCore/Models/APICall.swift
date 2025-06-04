import Foundation

enum HttpMethod: String {
    case GET
    case POST
}

protocol APICall {
    var path: String? { get }
    var method: HttpMethod { get }
    var headers: [String: String]? { get }
}

enum APIError: Swift.Error {
    case invalidURL
    case httpCode(HTTPCode)
    case unexpectedResponse
}

extension APIError: LocalizedError {
    var errorDescription: String? {
        switch self {
        case .invalidURL: return "Invalid URL"
        case let .httpCode(code): return "Unexpected HTTP code: \(code)"
        case .unexpectedResponse: return "Unexpected response from the server"
        }
    }
}

extension APICall {
    func urlRequest(baseURL: String, body: Data? = nil) throws -> URLRequest {
        guard let validPath = path,
              let url = URL(string: baseURL + validPath) else {
            throw APIError.invalidURL
        }
        var request = URLRequest(url: url)
        request.httpMethod = method.rawValue
        request.allHTTPHeaderFields = headers
        if let suppliedBody = body {
            request.httpBody = suppliedBody
        }
        return request
    }
}

typealias HTTPCode = Int
typealias HTTPCodes = Range<HTTPCode>

extension HTTPCodes {
    static let success = 200 ..< 300
}
