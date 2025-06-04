use chrono::{DateTime, Utc};
use common::{
    api::forms::{PostJournalistForm, PostJournalistIdPublicKeyForm},
    protocol::keys::{
        anchor_org_pk, verify_journalist_provisioning_pk, JournalistIdKeyPair,
        UntrustedAnchorOrganizationPublicKey, UntrustedJournalistIdKeyPair,
        UntrustedJournalistProvisioningPublicKey,
    },
};
use sqlx::{Executor, Sqlite, Transaction};

use crate::{key_rows::SeedInfoRow, ReplacementStrategy};

pub(crate) async fn insert_vault_setup_bundle(
    tx: &mut Transaction<'_, Sqlite>,
    provisioning_pk_id: i64,
    id_key_pair: &JournalistIdKeyPair,
    pk_upload_form_json: PostJournalistIdPublicKeyForm,
    register_journalist_form: Option<PostJournalistForm>,
    replacement_strategy: ReplacementStrategy,
) -> anyhow::Result<()> {
    let key_pair_json = serde_json::to_string(&id_key_pair.to_untrusted())?;
    let pk_upload_form_json = serde_json::to_string(&pk_upload_form_json)?;
    let register_journalist_form_json = register_journalist_form
        .map(|form| serde_json::to_string(&form))
        .transpose()?;

    // Full match statement without default fallback so the type system will alert us if we add a new variant
    match replacement_strategy {
        ReplacementStrategy::Replace => {
            sqlx::query!("DELETE FROM vault_setup_bundle")
                .execute(&mut **tx)
                .await?;
        }
        ReplacementStrategy::Keep => {
            // Don't delete the existing vault. The INSERT query below
            // will violate a constraint if there's already a setup bundle
            // which will emit an error
        }
    }

    sqlx::query!(
        r#"
            INSERT INTO vault_setup_bundle
                (
                    provisioning_pk_id,
                    pk_upload_form_json,
                    keypair_json,
                    register_journalist_form_json
                )
            VALUES
                (?1, ?2, ?3, ?4)
        "#,
        provisioning_pk_id,
        pk_upload_form_json,
        key_pair_json,
        register_journalist_form_json,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub(crate) async fn get_vault_setup_bundle<'a, E>(
    conn: E,
    now: DateTime<Utc>,
) -> anyhow::Result<Option<SeedInfoRow>>
where
    E: Executor<'a, Database = Sqlite>,
{
    sqlx::query!(
        r#"
            SELECT
                vault_setup_bundle.id                AS "id: i64",
                provisioning_pk_id                   AS "provisioning_pk_id: i64",
                pk_upload_form_json                  AS "pk_upload_form_json: String",
                keypair_json                         AS "keypair_json: String",
                register_journalist_form_json        AS "register_journalist_form_json: String",
                journalist_provisioning_pks.pk_json  AS "provisioning_pk_json: String",
                anchor_organization_pks.pk_json     AS "org_pk_json: String"
            FROM vault_setup_bundle
            JOIN journalist_provisioning_pks
                ON journalist_provisioning_pks.id = vault_setup_bundle.provisioning_pk_id
            JOIN anchor_organization_pks
                ON anchor_organization_pks.id = journalist_provisioning_pks.organization_pk_id
        "#
    )
    .fetch_optional(conn)
    .await?
    .map(|row| -> anyhow::Result<SeedInfoRow> {
        let org_pk =
            serde_json::from_str::<UntrustedAnchorOrganizationPublicKey>(&row.org_pk_json)?;
        let org_pk = anchor_org_pk(&org_pk, now)?.into_non_anchor();

        let provisioning_pk = serde_json::from_str::<UntrustedJournalistProvisioningPublicKey>(
            &row.provisioning_pk_json,
        )?;
        let provisioning_pk = verify_journalist_provisioning_pk(&provisioning_pk, &org_pk, now)?;

        let pk_upload_form =
            serde_json::from_str::<PostJournalistIdPublicKeyForm>(&row.pk_upload_form_json)?;

        let key_pair = serde_json::from_str::<UntrustedJournalistIdKeyPair>(&row.keypair_json)?;
        let key_pair = key_pair.to_trusted(&provisioning_pk, now)?;

        let register_journalist_form = row
            .register_journalist_form_json
            .map(|json: String| serde_json::from_str(&json))
            .transpose()?;

        anyhow::Ok(SeedInfoRow::new(
            row.provisioning_pk_id,
            pk_upload_form,
            key_pair,
            register_journalist_form,
        ))
    })
    .transpose()
}

pub(crate) async fn delete_vault_setup_bundle<'a, E>(conn: E) -> anyhow::Result<()>
where
    E: Executor<'a, Database = Sqlite>,
{
    sqlx::query!("DELETE FROM vault_setup_bundle")
        .execute(conn)
        .await?;

    Ok(())
}
