import Foundation

extension Array {
    // Safely lookup an index that might be out of bounds,
    // returning nil if it does not exist
    func get<T>(index: Int) -> T? {
        if index >= 0, index < count {
            return self[index] as? T
        } else {
            return nil
        }
    }
}

extension Array {
    /// Splits an Array into `size` length chunks
    /// note if the array cannot be chunked evenly, the final sequece will contain the remaing elements
    /// - Parameter size: the length of the chunk you want to split by
    /// - Returns: An Array of Arrays
    func chunked(into size: Int) -> [[Element]] {
        return stride(from: 0, to: count, by: size).map {
            Array(self[$0 ..< Swift.min($0 + size, count)])
        }
    }

    func splitAt(offset: Int) -> ([Element], [Element]) {
        assert(offset >= 0)
        assert(offset < count)
        return (
            Array(self[0 ..< offset]),
            Array(self[offset ..< count])
        )
    }

    /// This checks the array contains exactly the items in the comparision array.
    func containsExactly<T: Equatable>(_ array: [T]) -> Bool {
        let contains = allSatisfy { item in array.contains(where: { $0 == item as! T }) }
        let count = self.count == array.count
        return contains && count
    }
}
