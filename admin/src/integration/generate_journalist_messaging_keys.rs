use chrono::{DateTime, Utc};
use common::{
    crypto::keys::serde::StorableKeyMaterial,
    protocol::keys::{
        generate_journalist_messaging_key_pair, load_anchor_org_pks, load_journalist_id_key_pairs,
        load_journalist_provisioning_key_pairs_with_parent_org_pks,
    },
};
use std::path::Path;

/// Generates a journalist messaging key for use in integration tests.
///
/// This function is **not safe for production**.
/// It assumes test-only key generation and disk persistence.
pub async fn generate_journalist_messaging_keys_for_integration_test(
    keys_path: impl AsRef<Path>,
    now: DateTime<Utc>,
) -> anyhow::Result<()> {
    let keys_path = keys_path.as_ref();

    // Load organization and journalist key pairs
    let org_pks = load_anchor_org_pks(keys_path, now)?;
    let provisioning_pks =
        load_journalist_provisioning_key_pairs_with_parent_org_pks(keys_path, &org_pks, now)?;

    let (_, provisioning_key_pair) = provisioning_pks.first().ok_or_else(|| {
        anyhow::anyhow!(
            "No journalist provisioning key pairs found in {:?}",
            keys_path
        )
    })?;

    let public_key = provisioning_key_pair.public_key();
    let id_key_pairs = load_journalist_id_key_pairs(keys_path, public_key, now)?;

    let latest_id_key_pair = id_key_pairs.first().ok_or_else(|| {
        anyhow::anyhow!("No journalist ID key pairs found, cannot generate messaging key pair")
    })?;

    let messaging_key_pair = generate_journalist_messaging_key_pair(latest_id_key_pair, now);
    messaging_key_pair.to_untrusted().save_to_disk(keys_path)?;

    Ok(())
}
