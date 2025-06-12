use chrono::{DateTime, Utc};
use common::protocol::keys::{
    anchor_org_pk, AnchorOrganizationPublicKey, UntrustedAnchorOrganizationPublicKey,
};
use sqlx::SqliteConnection;

use crate::key_rows::OrganizationPublicKeyRow;

pub(crate) async fn insert_org_pk(
    conn: &mut SqliteConnection,
    org_pk: &AnchorOrganizationPublicKey,
    now: DateTime<Utc>,
) -> anyhow::Result<()> {
    let org_pk = org_pk.to_untrusted();
    let pk_json = serde_json::to_string(&org_pk)?;

    sqlx::query!(
        r#"
            INSERT OR IGNORE INTO anchor_organization_pks (pk_json, added_at)
            SELECT ?1, ?2
            WHERE NOT EXISTS (
                SELECT pk_json FROM anchor_organization_pks
                WHERE json_extract(pk_json, '$.key') = json_extract(?1, '$.key')
                AND json_extract(pk_json, '$.certificate') = json_extract(?1, '$.certificate')
                AND json_extract(pk_json, '$.not_valid_after') = json_extract(?1, '$.not_valid_after')
            );
        "#,
        pk_json,
        now
    )
    .execute(conn)
    .await?;

    Ok(())
}

pub(crate) async fn org_pks(
    conn: &mut SqliteConnection,
    now: DateTime<Utc>,
) -> anyhow::Result<impl Iterator<Item = OrganizationPublicKeyRow>> {
    let pks = sqlx::query!("SELECT id, pk_json FROM anchor_organization_pks")
        .fetch_all(conn)
        .await?
        .into_iter()
        .flat_map(move |row| {
            let pk = serde_json::from_str::<UntrustedAnchorOrganizationPublicKey>(&row.pk_json)?;
            let pk = anchor_org_pk(&pk, now)?;

            let pk_row = OrganizationPublicKeyRow::new(row.id, pk);
            anyhow::Ok(pk_row)
        });

    Ok(pks)
}

pub(crate) async fn delete_expired_org_pks(
    conn: &mut SqliteConnection,
    now: DateTime<Utc>,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
            DELETE FROM anchor_organization_pks
            WHERE pk_json->>'not_valid_after' < $1;
        "#,
        now
    )
    .execute(conn)
    .await?;

    Ok(())
}
