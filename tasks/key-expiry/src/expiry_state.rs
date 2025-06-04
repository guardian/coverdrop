pub enum ExpiryState<T> {
    /// The time remaining on this key is within normal range
    Nominal,
    /// This key is expiring soon and should have been rotated already
    ShouldHaveRotated(T),
    /// This key has already expired. This might be a big problem depending on the key.
    Expired,
}
