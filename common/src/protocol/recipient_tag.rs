use crate::api::models::journalist_id::JournalistIdentity;
use crate::protocol::constants::RECIPIENT_TAG_LEN;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use ts_rs::TS;

/// A type alias for the tag at the beginning of the serialized message that indicates the intended
/// recipient (or the fact it's a cover message).
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Hash, TS)]
#[serde(transparent, deny_unknown_fields)]
pub struct RecipientTag(
    #[serde(with = "hex")]
    #[ts(as = "String")]
    [u8; RECIPIENT_TAG_LEN],
);

pub const RECIPIENT_TAG_FOR_COVER: RecipientTag = RecipientTag::new([0u8; RECIPIENT_TAG_LEN]);

impl RecipientTag {
    pub const fn new(tag: [u8; RECIPIENT_TAG_LEN]) -> Self {
        Self(tag)
    }

    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        if bytes.len() != RECIPIENT_TAG_LEN {
            anyhow::bail!("invalid recipient tag length: {}", bytes.len());
        }

        let mut tag = [0; RECIPIENT_TAG_LEN];
        tag.copy_from_slice(bytes);

        Ok(RecipientTag(tag))
    }

    pub fn from_journalist_id(journalist_id: &JournalistIdentity) -> RecipientTag {
        // note: the hash operation here does not need any particular security properties (it should
        // just map pseudo randomly into the output domain to avoid collisions)
        let mut hasher = Sha256::new();

        // note: when reading the identifier un the `load_all_journalist_public_keys_from_disk`
        // operation only the part between `journalist_` and `.pub.json` is used for forming the
        // identifier.
        hasher.update(journalist_id.as_bytes());
        let hash = hasher.finalize();

        let mut truncated_hash = [0; RECIPIENT_TAG_LEN];
        truncated_hash.copy_from_slice(&hash[..RECIPIENT_TAG_LEN]);

        let tag = RecipientTag(truncated_hash);
        // note: this is virtually impossible to happen (2^-32); if it happens, the respective
        // journalist can be given a different identifier
        assert_ne!(
            tag, RECIPIENT_TAG_FOR_COVER,
            "collision with recipient tag for cover"
        );

        tag
    }
}

impl AsRef<[u8]> for RecipientTag {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
