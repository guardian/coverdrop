use chrono::{DateTime, Duration, Utc};
use common::{
    crypto::keys::{encryption::UnsignedEncryptionKeyPair, signing::UnsignedSigningKeyPair},
    epoch::Epoch,
    protocol::{
        constants::{
            JOURNALIST_ID_KEY_VALID_DURATION, JOURNALIST_MSG_KEY_VALID_DURATION,
            JOURNALIST_PROVISIONING_KEY_VALID_DURATION, ORGANIZATION_KEY_VALID_DURATION,
        },
        keys::{anchor_org_pk, generate_organization_key_pair, AnchorOrganizationPublicKeys},
    },
};
use sqlx::{pool::PoolConnection, Sqlite};

use crate::{
    id_key_queries::{insert_registered_id_key_pair, published_id_key_pairs},
    msg_key_queries::{candidate_msg_key_pair, insert_candidate_msg_key_pair},
    org_key_queries::insert_org_pk,
    provisioning_key_queries::{
        delete_expired_provisioning_pks, insert_journalist_provisioning_pk,
        journalist_provisioning_pks,
    },
};

#[sqlx::test]
async fn test_cascading_deletes(mut conn: PoolConnection<Sqlite>) -> sqlx::Result<()> {
    let now: DateTime<Utc> = "2025-07-28T10:30:00Z".parse().unwrap();

    let org_key_pair = generate_organization_key_pair(now);
    let trusted_org_pk = org_key_pair.to_public_key();
    let untrusted_org_pk = trusted_org_pk.to_untrusted();

    let anchor_org_pk =
        anchor_org_pk(&untrusted_org_pk.to_tofu_anchor(), now).expect("Make org pk");
    insert_org_pk(&mut conn, &anchor_org_pk, now).await.unwrap();
    let org_pks = vec![anchor_org_pk];
    let trust_anchors = AnchorOrganizationPublicKeys::new(org_pks.clone());

    // insert a provisioning key that will outlive its parent
    let provisioning_key_not_valid_after =
        now + ORGANIZATION_KEY_VALID_DURATION + JOURNALIST_PROVISIONING_KEY_VALID_DURATION;
    let journalist_provisioning_key_pair = UnsignedSigningKeyPair::generate()
        .to_signed_key_pair(&org_key_pair, provisioning_key_not_valid_after);
    insert_journalist_provisioning_pk(
        &mut conn,
        &trusted_org_pk,
        &journalist_provisioning_key_pair.to_public_key(),
        now,
    )
    .await
    .unwrap();

    let mut db_journalist_provisioning_pks =
        journalist_provisioning_pks(&mut conn, now, trust_anchors.clone())
            .await
            .unwrap();
    let db_provisioning_pk = db_journalist_provisioning_pks.next().unwrap();

    // insert an id key that will outlive its parent
    let id_key_not_valid_after =
        provisioning_key_not_valid_after + JOURNALIST_ID_KEY_VALID_DURATION;
    let journalist_id_key_pair = UnsignedSigningKeyPair::generate()
        .to_signed_key_pair(&journalist_provisioning_key_pair, id_key_not_valid_after);
    let created_at = now;
    let published_at = now;
    let id_key_epoch = Epoch(0);
    insert_registered_id_key_pair(
        &mut conn,
        db_provisioning_pk.id,
        &journalist_id_key_pair,
        created_at,
        published_at,
        id_key_epoch,
    )
    .await
    .unwrap();
    let mut journalist_id_key_pairs = published_id_key_pairs(&mut conn, now, trust_anchors.clone())
        .await
        .unwrap();
    let db_id_key_pair_row = journalist_id_key_pairs.next().unwrap();

    // insert a msg key that will outlive its parent
    let msg_key_not_valid_after = id_key_not_valid_after + JOURNALIST_MSG_KEY_VALID_DURATION;
    let journalist_msg_key_pair = UnsignedEncryptionKeyPair::generate()
        .to_signed_key_pair(&journalist_id_key_pair, msg_key_not_valid_after);
    insert_candidate_msg_key_pair(
        &mut conn,
        db_id_key_pair_row.key_pair.public_key(),
        &journalist_msg_key_pair,
        now,
        trust_anchors.clone(),
    )
    .await
    .unwrap();
    let journalist_msg_key_pairs = candidate_msg_key_pair(&mut conn, now, trust_anchors.clone())
        .await
        .unwrap();
    assert!(journalist_msg_key_pairs.is_some());

    let after_provisioning_key_expiry = provisioning_key_not_valid_after + Duration::days(1);
    delete_expired_provisioning_pks(&mut conn, after_provisioning_key_expiry)
        .await
        .unwrap();

    // provisioning key deleted
    let journalist_provisioning_pks_after = journalist_provisioning_pks(
        &mut conn,
        after_provisioning_key_expiry,
        trust_anchors.clone(),
    )
    .await
    .unwrap();
    assert_eq!(journalist_provisioning_pks_after.count(), 0);

    // id key deleted
    let journalist_id_key_pairs_after = published_id_key_pairs(
        &mut conn,
        after_provisioning_key_expiry,
        trust_anchors.clone(),
    )
    .await
    .unwrap();
    assert_eq!(journalist_id_key_pairs_after.count(), 0);

    // msg key deleted
    let msg_keys_after = candidate_msg_key_pair(
        &mut conn,
        after_provisioning_key_expiry,
        trust_anchors.clone(),
    )
    .await
    .unwrap();
    assert!(msg_keys_after.is_none());

    Ok(())
}
