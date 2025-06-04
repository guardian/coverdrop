use std::collections::HashMap;

use chrono::{DateTime, Utc};
use common::protocol::keys::{
    anchor_org_pk, verify_journalist_provisioning_pk, JournalistIdPublicKey,
    JournalistIdPublicKeyFamily, JournalistIdPublicKeyFamilyList, JournalistProvisioningPublicKey,
    OrganizationPublicKey, UntrustedAnchorOrganizationPublicKey, UntrustedJournalistIdKeyPair,
    UntrustedJournalistMessagingKeyPair, UntrustedJournalistProvisioningPublicKey,
};
use itertools::Itertools as _;
use sqlx::SqliteConnection;

/// Get the ID public key families for debugging purposes
pub(crate) async fn id_pk_families(
    conn: &mut SqliteConnection,
    now: DateTime<Utc>,
) -> anyhow::Result<JournalistIdPublicKeyFamilyList> {
    let rows = sqlx::query!(
        r#"
            SELECT
                journalist_msg_key_pairs.id           AS "msg_key_pair_id: i64",
                journalist_msg_key_pairs.key_pair_json AS "msg_key_pair_json: String",
                journalist_msg_key_pairs.added_at     AS "msg_key_pair_added_at: DateTime<Utc>",
                journalist_id_key_pairs.id            AS "id_key_pair_id: i64",
                journalist_id_key_pairs.key_pair_json  AS "id_key_pair_json: String",
                journalist_id_key_pairs.added_at      AS "id_key_pair_added_at: DateTime<Utc>",
                journalist_provisioning_pks.id        AS "provisioning_pk_id: i64",
                journalist_provisioning_pks.pk_json   AS "provisioning_pk_json: String",
                anchor_organization_pks.id           AS "org_pk_id: i64",
                anchor_organization_pks.pk_json      AS "org_pk_json: String"
            FROM journalist_msg_key_pairs
            JOIN journalist_id_key_pairs
                ON journalist_id_key_pairs.id = journalist_msg_key_pairs.id_key_pair_id
            JOIN journalist_provisioning_pks
                ON journalist_provisioning_pks.id = journalist_id_key_pairs.provisioning_pk_id
            JOIN anchor_organization_pks
                ON anchor_organization_pks.id = journalist_provisioning_pks.organization_pk_id
        "#
    )
    .fetch_all(conn)
    .await?;

    // Gather all org pks
    let anchor_org_pks = rows
        .iter()
        .map(|row| (row.org_pk_id, &row.org_pk_json))
        .unique()
        .map(|(org_pk_id, org_pk_json)| {
            let org_pk = serde_json::from_str::<UntrustedAnchorOrganizationPublicKey>(org_pk_json)?;
            let anchor_org_pk = anchor_org_pk(&org_pk, now)?.into_non_anchor();

            anyhow::Ok((org_pk_id, anchor_org_pk))
        })
        .collect::<anyhow::Result<HashMap<i64, OrganizationPublicKey>>>()?;

    // Gather all provisioning pks
    let provisioning_pks = rows
        .iter()
        .map(|row| {
            (
                row.org_pk_id,
                row.provisioning_pk_id,
                &row.provisioning_pk_json,
            )
        })
        .unique()
        .map(|(org_pk_id, provisioning_pk_id, provisioning_pk_json)| {
            let Some(org_pk) = anchor_org_pks.get(&org_pk_id) else {
                anyhow::bail!("No trusted org PK for ID {}", org_pk_id);
            };

            let provisioning_pk = serde_json::from_str::<UntrustedJournalistProvisioningPublicKey>(
                provisioning_pk_json,
            )?;

            let provisioning_pk = verify_journalist_provisioning_pk(&provisioning_pk, org_pk, now)?;

            anyhow::Ok((provisioning_pk_id, provisioning_pk))
        })
        .collect::<anyhow::Result<HashMap<i64, JournalistProvisioningPublicKey>>>()?;

    let id_pks = rows
        .iter()
        .map(|row| {
            (
                row.provisioning_pk_id,
                row.id_key_pair_id,
                &row.id_key_pair_json,
            )
        })
        .unique()
        .map(|(provisioning_pk_id, id_key_pair_id, id_key_pair_json)| {
            let Some(provisioning_pk) = provisioning_pks.get(&provisioning_pk_id) else {
                anyhow::bail!("No provisioning PK for ID {}", provisioning_pk_id);
            };

            let id_key_pair =
                serde_json::from_str::<UntrustedJournalistIdKeyPair>(id_key_pair_json)?;

            let id_pk = id_key_pair
                .to_trusted(provisioning_pk, now)?
                .to_public_key();

            anyhow::Ok((id_key_pair_id, id_pk))
        })
        .collect::<anyhow::Result<HashMap<i64, JournalistIdPublicKey>>>()?;

    let families = id_pks
        .into_iter()
        .map(|(id_pk_id, id_pk)| {
            let msg_pks = rows
                .iter()
                .flat_map(|row| {
                    if row.id_key_pair_id == id_pk_id {
                        let msg_key_pair = serde_json::from_str::<
                            UntrustedJournalistMessagingKeyPair,
                        >(&row.msg_key_pair_json)?;

                        msg_key_pair
                            .to_trusted(&id_pk, now)
                            .map(|msg_key_pair| msg_key_pair.public_key().clone())
                    } else {
                        anyhow::bail!("Wrong messaging key pair to validate")
                    }
                })
                .collect();
            JournalistIdPublicKeyFamily::new(id_pk, msg_pks)
        })
        .collect();

    Ok(JournalistIdPublicKeyFamilyList::new(families))
}
