import Foundation

protocol WebRepository {
    var urlSession: URLSession { get }
    var baseUrl: String { get }
}

extension WebRepository {
    func call<Value>(endpoint: APICall, httpCodes _: HTTPCodes = .success) async throws -> Value
        where Value: Decodable {
        do {
            let request = try endpoint.urlRequest(baseURL: baseUrl)
            let (data, response) = try await urlSession.data(for: request)
            guard let httpResponse = response as? HTTPURLResponse,
                  HTTPCodes.success.contains(httpResponse.statusCode) else {
                throw URLError(.badServerResponse)
            }
            let decoded = try JSONDecoder().decode(Value.self, from: data)
            return decoded
        } catch {
            throw URLError(.badServerResponse)
        }
    }

    func post(endpoint: APICall, httpCodes _: HTTPCodes = .success, body: Data?) async throws -> HTTPURLResponse {
        let request = try endpoint.urlRequest(baseURL: baseUrl, body: body)
        let (_, response) = try await urlSession.data(for: request)
        Debug.println("Made successful post to \(String(describing: request.url))")
        guard let httpResponse = response as? HTTPURLResponse,
              HTTPCodes.success.contains(httpResponse.statusCode) else {
            throw URLError(.badServerResponse)
        }
        return httpResponse
    }
}

// swiftlint:disable type_name
protocol CacheableWebRepository<T>: WebRepository {
    associatedtype T: Codable
    func get(params: [String: String]?) async throws -> T
}

// swiftlint:enable type_name
