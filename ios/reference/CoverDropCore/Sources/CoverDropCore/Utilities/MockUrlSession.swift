import Foundation

class URLProtocolMock: URLProtocol {
    /// Dictionary maps URLs to tuples of error, data, and response
    static var mockURLs = [URL?: MockResponse]()

    override class func canInit(with _: URLRequest) -> Bool {
        // Handle all types of requests
        return true
    }

    override class func canonicalRequest(for request: URLRequest) -> URLRequest {
        // Required to be implemented here. Just return what is passed
        return request
    }

    override func startLoading() {
        if let url = request.url {
            if let mockResponse = URLProtocolMock.mockURLs[url] {
                // We have a mock response specified so return it.
                client?.urlProtocol(self, didReceive: mockResponse.response, cacheStoragePolicy: .notAllowed)

                // We have mocked data specified so return it.
                client?.urlProtocol(self, didLoad: mockResponse.data)

                // We have a mocked error so return it.
                if let errorStrong = mockResponse.error {
                    client?.urlProtocol(self, didFailWithError: errorStrong)
                }
            } else {
                client?.urlProtocol(self, didFailWithError: APIError.unexpectedResponse)
            }
        }

        // Send the signal that we are done returning our mock response
        client?.urlProtocolDidFinishLoading(self)
    }

    override func stopLoading() {
        // Required to be implemented. Do nothing here.
    }
}
