import Foundation

class MockURLProtocol: URLProtocol {
    override class func canInit(with _: URLRequest) -> Bool {
        // To check if this protocol can handle the given request.
        return true
    }

    override class func canonicalRequest(for request: URLRequest) -> URLRequest {
        // Here you return the canonical version of the request but most of the time you pass the orignal one.
        return request
    }

    static var requestHandler: ((URLRequest) throws -> (HTTPURLResponse, Data?))?

    override func startLoading() {
        guard let handler = MockURLProtocol.requestHandler else {
            fatalError("Handler is unavailable.")
        }

        do {
            let (response, data) = try handler(request)
            client?.urlProtocol(self, didReceive: response, cacheStoragePolicy: .notAllowed)

            if let data = data {
                client?.urlProtocol(self, didLoad: data)
            }

            client?.urlProtocolDidFinishLoading(self)
        } catch {
            client?.urlProtocol(self, didFailWithError: error)
        }
    }

    override func stopLoading() {
        // This is called if the request gets canceled or completed.
    }
}
