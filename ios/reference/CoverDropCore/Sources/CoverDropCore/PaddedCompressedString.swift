import Foundation
import Gzip
import Sodium

public enum PaddedCompressedStringError: Error {
    case compressedStringTooLong
    case paddedCompressedStringTooLong
    case gzipCompressionError
    case invaidUTF8StringConversion
    case decompressionRatioTooHigh
    case incorrectByteLength
    case failedToGenerateRandomBytes
}

public struct PaddedCompressedString: Equatable, Encryptable {
    public func asUnencryptedBytes() -> [UInt8] {
        value
    }

    public static func fromUnencryptedBytes(bytes: [UInt8]) throws -> PaddedCompressedString {
        try PaddedCompressedString.fromUncheckedBytes(bytes: bytes)
    }

    public let value: [UInt8]

    // this is a UInt16 expressed as maximum number of bytes, which is 2
    static let headerSize = 2

    /// The first step of constructing a `PaddedCompressedString`, but can also be called externally to check whether a
    /// user's message exceeds the allowed size
    @discardableResult
    public static func compressCheckingLength(from text: String) throws -> (compressedData: Data, compressedSize: Int) {
        let padToSize = Constants.messagePaddingLen

        guard let data = text.data(using: .utf8) else {
            throw PaddedCompressedStringError.invaidUTF8StringConversion
        }
        let compressedData: Data = try data.gzipped()

        let compressedSize: Int = compressedData.count
        // Check to make sure the compressed size is not greater that the size we want to pad to.
        if compressedSize + headerSize > padToSize {
            throw PaddedCompressedStringError.compressedStringTooLong
        }
        return (compressedData: compressedData, compressedSize: compressedSize)
    }

    public static func fromString(text: String) throws -> PaddedCompressedString {
        let padToSize = Constants.messagePaddingLen

        let (compressedData, compressedSize) = try compressCheckingLength(from: text)

        // This taking the integer value of compressed size (ie 46) and representing it as a a UInt8 ,
        // as the `padToSize` is only a UInt16 it can never be more that 2 bytes,
        // so we pad it just in case it was able to represent the compressed size as a single byte.
        let source = UInt16(compressedSize).bigEndian
        let header = withUnsafeBytes(of: source) { Data($0) }

        var buffer: [UInt8] = Array(header)

        // append the compressed data
        buffer.append(contentsOf: Array(compressedData))

        // pad with random bytes to meet the specified length requirement
        guard let paddingBytes = Sodium().randomBytes.buf(length: padToSize - buffer.count) else {
            throw PaddedCompressedStringError.failedToGenerateRandomBytes
        }
        buffer.append(contentsOf: Array(paddingBytes))

        return PaddedCompressedString(value: buffer)
    }

    public func toString() throws -> String {
        let bytes = value
        let headerSize = PaddedCompressedString.headerSize
        let compressedSizeBytes = Array(value.prefix(headerSize))
        let compressedSize: UInt16 = compressedSizeBytes.withUnsafeBytes { $0.load(as: UInt16.self) }
        let hostEndianSize = Int(UInt16(bigEndian: compressedSize))
        let endOfCompressedBytes = hostEndianSize + headerSize

        let compressedBytes = Array(bytes[headerSize ... endOfCompressedBytes])

        let decoded = try Data(compressedBytes).gunzipped()
        // The maximum compression ratio is ~1000:1. This is our (256 byte) messages would not
        // decode to output larger than 256 KiB. Nevertheless, we assume that everything with a
        // compression ratio larger than 100:1 is suspicious for natural text and we drop it.
        // See: https://github.com/guardian/coverdrop/issues/112
        let decompressionRatio = decoded.count / hostEndianSize
        if decompressionRatio > 100 {
            throw PaddedCompressedStringError.decompressionRatioTooHigh
        }

        return try String.fromUnencryptedBytes(bytes: Array(decoded))
    }

    public static func fromUncheckedBytes(bytes: [UInt8]) throws -> PaddedCompressedString {
        if bytes.count != Constants.messagePaddingLen {
            throw PaddedCompressedStringError.incorrectByteLength
        }
        return PaddedCompressedString(value: bytes)
    }

    public func totalLength() -> Int {
        return value.count
    }

    public func compressedDataLen() -> Int {
        return value.count - PaddedCompressedString.headerSize
    }

    public func paddingLength() -> Int {
        return totalLength() - compressedDataLen() - PaddedCompressedString.headerSize
    }

    public func fillLevel() throws -> Float32 {
        let maxCompressedDataLen = totalLength() - PaddedCompressedString.headerSize
        return Float32(compressedDataLen() / maxCompressedDataLen)
    }
}
