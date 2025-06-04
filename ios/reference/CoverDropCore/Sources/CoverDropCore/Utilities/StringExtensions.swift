public extension [String] {
    func joinTo(separator: String, prefix: String, suffix: String) -> String {
        return prefix + self.joined(separator: separator) + suffix
    }
}
