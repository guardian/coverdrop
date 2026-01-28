use std::sync::OnceLock;

use chrono::{DateTime, Utc};
use common::{
    clap::Stage,
    protocol::keys::{
        anchor_org_pk, AnchorOrganizationPublicKey, UntrustedAnchorOrganizationPublicKey,
    },
};

static PRODUCTION_ANCHORS: OnceLock<Vec<UntrustedAnchorOrganizationPublicKey>> = OnceLock::new();
static STAGING_ANCHORS: OnceLock<Vec<UntrustedAnchorOrganizationPublicKey>> = OnceLock::new();
static DEVELOPMENT_ANCHORS: OnceLock<Vec<UntrustedAnchorOrganizationPublicKey>> = OnceLock::new(); // used by integration tests

pub fn get_trust_anchors(
    stage: &Stage,
    now: DateTime<Utc>,
) -> anyhow::Result<Vec<AnchorOrganizationPublicKey>> {
    let untrusted_trust_anchors = match stage {
        Stage::Production => PRODUCTION_ANCHORS.get_or_init(|| {
            serde_json::from_str(include_str!("../production.json"))
                .expect("Failed to parse production.json")
        }),
        Stage::Staging => STAGING_ANCHORS.get_or_init(|| {
            serde_json::from_str(include_str!("../staging.json"))
                .expect("Failed to parse staging.json")
        }),
        Stage::Development => DEVELOPMENT_ANCHORS.get_or_init(|| {
            serde_json::from_str(include_str!("../development.json"))
                .expect("Failed to parse development.json")
        }),
        _ => anyhow::bail!("Unsupported stage for trust anchors"),
    };

    let trust_anchors = untrusted_trust_anchors
        .iter()
        .flat_map(move |org_pk| {
            anchor_org_pk(org_pk, now).map_err(|e| {
                tracing::warn!(
                    "Failed to trust org_pk {}: {}",
                    hex::encode(org_pk.key.as_bytes()),
                    e,
                );
                e
            })
        })
        .collect();

    Ok(trust_anchors)
}
