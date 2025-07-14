use axum::{extract::State, Json};
use common::protocol::keys::{
    UntrustedAnchorOrganizationPublicKey, UntrustedCoverNodeIdPublicKey,
    UntrustedCoverNodeMessagingPublicKey, UntrustedUnregisteredCoverNodeIdPublicKey,
};
use serde::{Deserialize, Serialize};

use crate::key_state::KeyState;

#[derive(Serialize, Deserialize)]
pub struct CoverNodeIdentityPublicKeys {
    candidate_id_pk: Option<UntrustedUnregisteredCoverNodeIdPublicKey>,
    published_id_pks: Vec<UntrustedCoverNodeIdPublicKey>,
}

#[derive(Serialize, Deserialize)]
pub struct CoverNodeMessagingPublicKeys {
    candidate_msg_pk: Option<UntrustedCoverNodeMessagingPublicKey>,
    published_msg_pks: Vec<UntrustedCoverNodeMessagingPublicKey>,
}

#[derive(Serialize, Deserialize)]
pub struct CoverNodePublicKeys {
    anchor_org_pks: Vec<UntrustedAnchorOrganizationPublicKey>,
    identity_pks: CoverNodeIdentityPublicKeys,
    messaging_keys: CoverNodeMessagingPublicKeys,
}

pub async fn get_public_keys(State(key_state): State<KeyState>) -> Json<CoverNodePublicKeys> {
    let key_state = key_state.read().await;

    let anchor_org_pks = key_state
        .anchor_org_pks()
        .iter()
        .map(|anchor_org_pk| anchor_org_pk.to_untrusted())
        .collect();

    let published_id_pks = key_state
        .published_covernode_id_key_pairs()
        .iter()
        .map(|published_id_key_pair| published_id_key_pair.key_pair.public_key().to_untrusted())
        .collect();

    let candidate_id_pk = key_state
        .candidate_covernode_id_key_pair()
        .as_ref()
        .map(|candidate_id_key_pair| candidate_id_key_pair.public_key().to_untrusted());

    let published_msg_pks = key_state
        .published_covernode_msg_key_pairs()
        .iter()
        .map(|published_msg_key_pair| published_msg_key_pair.key_pair.public_key().to_untrusted())
        .collect();

    let candidate_msg_pk = key_state
        .candidate_covernode_msg_key_pair()
        .as_ref()
        .map(|candidate_msg_key_pair| candidate_msg_key_pair.public_key().to_untrusted());

    let identity_pks = CoverNodeIdentityPublicKeys {
        published_id_pks,
        candidate_id_pk,
    };

    let messaging_keys = CoverNodeMessagingPublicKeys {
        published_msg_pks,
        candidate_msg_pk,
    };

    Json(CoverNodePublicKeys {
        anchor_org_pks,
        identity_pks,
        messaging_keys,
    })
}
