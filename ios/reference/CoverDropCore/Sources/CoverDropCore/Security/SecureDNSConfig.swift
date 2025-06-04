import Foundation
import Network

// This config is from https://gist.github.com/sschizas/ed03571ed129a227947f482b51ffabc5

enum SecureDNSConfig: Hashable {
    case cloudflare

    var httpsURL: URL {
        switch self {
        case .cloudflare:
            return URL(string: "https://cloudflare-dns.com/dns-query")!
        }
    }

    var serverAddresses: [NWEndpoint] {
        switch self {
        case .cloudflare:
            return [
                NWEndpoint.hostPort(host: "1.1.1.1", port: 443),
                NWEndpoint.hostPort(host: "1.0.0.1", port: 443),
                NWEndpoint.hostPort(host: "2606:4700:4700::1111", port: 443),
                NWEndpoint.hostPort(host: "2606:4700:4700::1001", port: 443)
            ]
        }
    }
}
