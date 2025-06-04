use base64::{prelude::BASE64_STANDARD_NO_PAD, Engine};
use sodiumoxide::crypto::hash::sha512;

use super::keys::Ed25519PublicKey;

pub fn human_readable_digest(key: &Ed25519PublicKey) -> String {
    let digest = sha512::hash(key.as_bytes()).0;

    // Encode the first 16 bytes of the digest to base64
    let truncated_digest = &digest[..16];
    let base64 = BASE64_STANDARD_NO_PAD.encode(truncated_digest);
    assert_eq!(base64.len(), 22);

    // Chunk the base64 string into parts of 6 characters and join with spaces
    let chunked = base64
        .as_bytes()
        .chunks(6)
        .map(|chunk| std::str::from_utf8(chunk).unwrap())
        .collect::<Vec<&str>>()
        .join(" ");

    chunked
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_human_readable_digest() {
        let key_hex = "c941a9beed1c8c945c27b150b5aa725a6366f71900a5e93607ba93254fe8d585";
        let mut key_bytes = [0u8; 32];
        hex::decode_to_slice(key_hex, &mut key_bytes).unwrap();

        let key = Ed25519PublicKey::from_bytes(&key_bytes).unwrap();

        let actual = human_readable_digest(&key);
        assert_eq!(actual, "jdiH4c 9DO9cT kefiCh OXoQ");
    }
}
