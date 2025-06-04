import Foundation
import Sodium

/// Generates a random string of given `length` using alphanumeric charaters only

public func randomSaltBytes(length: Int) -> [UInt8]? {
    Sodium().randomBytes.buf(length: length)
}
