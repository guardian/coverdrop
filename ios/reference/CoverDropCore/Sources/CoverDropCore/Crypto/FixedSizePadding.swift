import Foundation

enum FixedSizedPaddingError: Error {
    case invalidSize(expected: Int, actual: Int)
    case invalidPaddedData
    case sizeMismatch(expected: Int, actual: Int)
    case targetSizeTooLarge(actual: Int, maximum: Int)
    case dataLengthExceedsAvailableSpace(dataLength: Int, availableSpace: Int)
}

class FixedSizedPadding {
    private let targetSize: Int
    private let bytes: [UInt8]

    /// Maximum target size: UInt32.max (~4 GiB)
    static let maxTargetSize = Int(UInt32.max)

    /// Creates a new FixedSizedPadding instance
    /// - Parameters:
    ///   - targetSize: The total size including the 4-byte header (must be <= UInt32.max)
    ///   - bytes: The actual data bytes (must be at most targetSize - 4)
    /// - Throws: FixedSizedPaddingError if validation fails
    init(targetSize: Int, bytes: [UInt8]) throws {
        guard targetSize <= Self.maxTargetSize else {
            throw FixedSizedPaddingError.targetSizeTooLarge(
                actual: targetSize,
                maximum: Self.maxTargetSize
            )
        }

        let maxDataSize = targetSize - 4
        guard bytes.count <= maxDataSize else {
            throw FixedSizedPaddingError.invalidSize(
                expected: maxDataSize,
                actual: bytes.count
            )
        }

        self.targetSize = targetSize
        self.bytes = bytes
    }

    /// Returns the padded bytes with the 4-byte header
    /// - Returns: Bytes containing [4-byte data length][original bytes][zero padding]
    func paddedBytes() -> [UInt8] {
        var result: [UInt8] = []

        // Encode data length as 4-byte big-endian header
        let dataLength = UInt32(bytes.count)
        result.append(contentsOf: Self.encodeUInt32BigEndian(dataLength))

        // Append original bytes
        result.append(contentsOf: bytes)

        // Add zero padding to reach targetSize
        let paddingNeeded = targetSize - 4 - bytes.count
        if paddingNeeded > 0 {
            result.append(contentsOf: [UInt8](repeating: 0x00, count: paddingNeeded))
        }

        return result
    }

    /// Creates a FixedSizedPadding instance from padded bytes
    /// - Parameters:
    ///   - paddedData: The complete padded data including header
    ///   - targetSize: The expected total size (must be <= UInt32.max)
    /// - Returns: A new FixedSizedPadding instance
    /// - Throws: FixedSizedPaddingError if validation fails
    static func fromPaddedBytes(_ paddedData: [UInt8], targetSize: Int) throws -> FixedSizedPadding {
        // Validate target size is within limits
        guard targetSize <= maxTargetSize else {
            throw FixedSizedPaddingError.targetSizeTooLarge(
                actual: targetSize,
                maximum: maxTargetSize
            )
        }

        // Validate minimum size (at least 4 bytes for header)
        guard paddedData.count >= 4 else {
            throw FixedSizedPaddingError.invalidPaddedData
        }

        // Validate that the actual data size matches the expected target size
        guard paddedData.count == targetSize else {
            throw FixedSizedPaddingError.sizeMismatch(
                expected: targetSize,
                actual: paddedData.count
            )
        }

        // Extract data length header (first 4 bytes, big-endian)
        let dataLength = Int(decodeUInt32BigEndian(Array(paddedData[0 ..< 4])))

        // Validate data length doesn't exceed available space
        let availableSpace = targetSize - 4
        guard dataLength <= availableSpace else {
            throw FixedSizedPaddingError.dataLengthExceedsAvailableSpace(
                dataLength: dataLength,
                availableSpace: availableSpace
            )
        }

        // Extract the original bytes based on the data length
        let originalBytes = Array(paddedData[4 ..< (4 + dataLength)])

        return try FixedSizedPadding(targetSize: targetSize, bytes: originalBytes)
    }

    /// Access the original bytes (without padding)
    func getBytes() -> [UInt8] {
        return bytes
    }

    /// Access the target size
    func getTargetSize() -> Int {
        return targetSize
    }

    // MARK: - Helper Functions

    /// Encodes a UInt32 value as 4 bytes in big-endian order
    private static func encodeUInt32BigEndian(_ value: UInt32) -> [UInt8] {
        return [
            UInt8((value >> 24) & 0xFF),
            UInt8((value >> 16) & 0xFF),
            UInt8((value >> 8) & 0xFF),
            UInt8(value & 0xFF)
        ]
    }

    /// Decodes a UInt32 value from 4 bytes in big-endian order
    private static func decodeUInt32BigEndian(_ bytes: [UInt8]) -> UInt32 {
        precondition(bytes.count == 4, "Expected exactly 4 bytes for UInt32 decoding")
        return UInt32(bytes[0]) << 24 |
            UInt32(bytes[1]) << 16 |
            UInt32(bytes[2]) << 8 |
            UInt32(bytes[3])
    }
}
