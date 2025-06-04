pub trait PublicKey {
    /// Get a hex representation of the public key bytes
    fn public_key_hex(&self) -> String;
}
