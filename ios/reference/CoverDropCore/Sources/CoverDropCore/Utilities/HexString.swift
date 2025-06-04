import CryptoKit
import Foundation
import Sodium

/// These extensions add utils to `Digest` and `[UInt8` types to allow the
/// creation of a hex String from byte array and Digest
extension Digest {
    var bytes: [UInt8] { Array(makeIterator()) }
    var data: Data { Data(bytes) }

    var hexStr: String? {
        Sodium().utils.bin2hex(bytes)
    }
}

public extension [UInt8] {
    var hexStr: String? {
        Sodium().utils.bin2hex(self)
    }
}
