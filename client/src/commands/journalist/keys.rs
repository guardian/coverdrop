use common::protocol::keys::UserPublicKey;
use journalist_vault::JournalistVault;
use std::collections::BTreeMap;

pub type IdentifierAndKey<K> = (String, K);

// TODO consider removing this, its pretty ugly.

/// Get the user keys that a given journalist knows about in a map from identifier to key.
pub async fn get_user_keys(
    vault: &JournalistVault,
) -> anyhow::Result<BTreeMap<String, IdentifierAndKey<UserPublicKey>>> {
    Ok(vault
        .user_keys()
        .await?
        .fold(BTreeMap::new(), |mut user_keys, key| {
            let identifier = hex::encode(key.key.as_bytes());
            user_keys.insert(identifier, ("user".to_owned(), key));
            user_keys
        }))
}
