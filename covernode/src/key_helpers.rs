use chrono::{DateTime, Utc};
use common::{
    crypto::keys::public_key::PublicKey,
    protocol::keys::{
        CoverNodeIdKeyPairWithEpoch, CoverNodeMessagingKeyPairWithEpoch,
        CoverNodeProvisioningPublicKey,
    },
};

use covernode_database::Database;

pub async fn get_and_verify_covernode_id_key_pairs(
    db: &Database,
    covernode_provisioning_pks: &[CoverNodeProvisioningPublicKey],
    now: DateTime<Utc>,
) -> anyhow::Result<Vec<CoverNodeIdKeyPairWithEpoch>> {
    let published_id_key_pairs = db.select_published_id_key_pairs().await?;

    let covernode_id_key_pairs = published_id_key_pairs
        .iter()
        .flat_map(|key_pair| {
            covernode_provisioning_pks
                .iter()
                .flat_map(|covernode_provisioning_pk| {
                    key_pair
                        .key_pair
                        .to_trusted(covernode_provisioning_pk, now)
                        .map(|signed_encrypted_key_pair| {
                            CoverNodeIdKeyPairWithEpoch::new(
                                signed_encrypted_key_pair,
                                key_pair.epoch,
                                key_pair.created_at,
                            )
                        })
                })
        })
        .inspect(|key_pair| {
            let public_key_hex = key_pair.key_pair.public_key_hex();
            tracing::debug!("Loaded CoverNode ID key pair: {}", public_key_hex);
        })
        .collect::<Vec<_>>();
    Ok(covernode_id_key_pairs)
}

pub async fn get_and_verify_covernode_msg_key_pairs(
    db: &Database,
    covernode_id_pks: &[CoverNodeIdKeyPairWithEpoch],
    now: DateTime<Utc>,
) -> anyhow::Result<Vec<CoverNodeMessagingKeyPairWithEpoch>> {
    let published_msg_key_pairs = db.select_published_msg_key_pairs().await?;

    let covernode_msg_key_pairs = published_msg_key_pairs
        .iter()
        .flat_map(|key_pair| {
            covernode_id_pks.iter().flat_map(|covernode_id_pk| {
                key_pair
                    .key_pair
                    .to_trusted(&covernode_id_pk.key_pair, now)
                    .map(|signed_encrypted_key_pair| {
                        CoverNodeMessagingKeyPairWithEpoch::new(
                            signed_encrypted_key_pair,
                            key_pair.epoch,
                            key_pair.created_at,
                        )
                    })
            })
        })
        .collect::<Vec<_>>();

    Ok(covernode_msg_key_pairs)
}
