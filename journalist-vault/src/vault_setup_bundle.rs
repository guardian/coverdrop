use anyhow::Context;
use chrono::{DateTime, Utc};
use common::{
    api::forms::{
        PostJournalistForm, PostJournalistIdPublicKeyForm, PostSentinelIdPublicKeyForm,
        PostSentinelProfileForm,
    },
    protocol::keys::{
        verify_journalist_provisioning_pk, AnchorOrganizationPublicKeys, JournalistIdKeyPair,
        SentinelIdKeyPair, UntrustedJournalistIdKeyPair, UntrustedJournalistProvisioningPublicKey,
        UntrustedSentinelIdKeyPair,
    },
};
use sqlx::{Executor, Sqlite, Transaction};

use crate::{key_rows::SeedInfoRow, ReplacementStrategy};

#[allow(clippy::too_many_arguments)]
pub(crate) async fn insert_vault_setup_bundle(
    tx: &mut Transaction<'_, Sqlite>,
    provisioning_pk_id: i64,
    id_key_pair: &JournalistIdKeyPair,
    journalist_id_pk_upload_form_json: PostJournalistIdPublicKeyForm,
    register_journalist_form: Option<PostJournalistForm>,
    sentinel_id_pk_upload_form_json: Option<PostSentinelIdPublicKeyForm>,
    sentinel_id_key_pair: Option<&SentinelIdKeyPair>,
    register_sentinel_profile_form: Option<PostSentinelProfileForm>,
    replacement_strategy: ReplacementStrategy,
) -> anyhow::Result<()> {
    let journalist_id_keypair_json = serde_json::to_string(&id_key_pair.to_untrusted())?;
    let journalist_id_pk_upload_form_json =
        serde_json::to_string(&journalist_id_pk_upload_form_json)?;
    let register_journalist_form_json = register_journalist_form
        .map(|form| serde_json::to_string(&form))
        .transpose()?;
    let sentinel_id_pk_upload_form_json = sentinel_id_pk_upload_form_json
        .map(|form| serde_json::to_string(&form))
        .transpose()?;
    let sentinel_id_keypair_json = sentinel_id_key_pair
        .map(|kp| serde_json::to_string(&kp.to_untrusted()))
        .transpose()?;
    let register_sentinel_profile_form_json = register_sentinel_profile_form
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
                    journalist_id_pk_upload_form_json,
                    journalist_id_keypair_json,
                    register_journalist_form_json,
                    sentinel_id_pk_upload_form_json,
                    sentinel_id_keypair_json,
                    register_sentinel_profile_form_json
                )
            VALUES
                (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        "#,
        provisioning_pk_id,
        journalist_id_pk_upload_form_json,
        journalist_id_keypair_json,
        register_journalist_form_json,
        sentinel_id_pk_upload_form_json,
        sentinel_id_keypair_json,
        register_sentinel_profile_form_json,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub(crate) async fn get_vault_setup_bundle<'a, E>(
    conn: E,
    now: DateTime<Utc>,
    trust_anchors: AnchorOrganizationPublicKeys,
) -> anyhow::Result<Option<SeedInfoRow>>
where
    E: Executor<'a, Database = Sqlite>,
{
    let org_pks_from_trust_anchors = trust_anchors.into_non_anchors();

    sqlx::query!(
        r#"
            SELECT
                vault_setup_bundle.id                          AS "id: i64",
                provisioning_pk_id                             AS "provisioning_pk_id: i64",
                journalist_id_pk_upload_form_json              AS "journalist_id_pk_upload_form_json: String",
                journalist_id_keypair_json                     AS "journalist_id_keypair_json: String",
                register_journalist_form_json                  AS "register_journalist_form_json: String",
                sentinel_id_pk_upload_form_json                AS "sentinel_id_pk_upload_form_json: String",
                sentinel_id_keypair_json                       AS "sentinel_id_keypair_json: String",
                register_sentinel_profile_form_json            AS "register_sentinel_profile_form_json: String",
                journalist_provisioning_pks.pk_json            AS "provisioning_pk_json: String"
            FROM vault_setup_bundle
            JOIN journalist_provisioning_pks
                ON journalist_provisioning_pks.id = vault_setup_bundle.provisioning_pk_id
        "#
    )
    .fetch_optional(conn)
    .await?
    .map(|row| -> anyhow::Result<SeedInfoRow> {
        let provisioning_pk = serde_json::from_str::<UntrustedJournalistProvisioningPublicKey>(
            &row.provisioning_pk_json,
        )?;

        // try to verify the provisioning pk against each trust anchor
        let provisioning_pk = org_pks_from_trust_anchors
            .iter()
            .find_map(|org_pk| {
                verify_journalist_provisioning_pk(&provisioning_pk, org_pk, now).ok()
            })
            .context(format!(
                "Could not verify vault_setup_bundle with id {}",
                row.id
            ))?;

        let pk_upload_form =
            serde_json::from_str::<PostJournalistIdPublicKeyForm>(&row.journalist_id_pk_upload_form_json)?;

        let key_pair = serde_json::from_str::<UntrustedJournalistIdKeyPair>(&row.journalist_id_keypair_json)?;
        let key_pair = key_pair.to_trusted(&provisioning_pk, now)?;

        let register_journalist_form = row
            .register_journalist_form_json
            .map(|json: String| serde_json::from_str(&json))
            .transpose()?;

        let sentinel_id_pk_upload_form = row
            .sentinel_id_pk_upload_form_json
            .map(|json: String| serde_json::from_str(&json))
            .transpose()?;

        let sentinel_id_key_pair = row
            .sentinel_id_keypair_json
            .map(|json: String| -> anyhow::Result<SentinelIdKeyPair> {
                let untrusted = serde_json::from_str::<UntrustedSentinelIdKeyPair>(&json)?;
                let trusted = untrusted.to_trusted(&provisioning_pk, now)?;
                Ok(trusted)
            })
            .transpose()?;

        let register_sentinel_profile_form = row
            .register_sentinel_profile_form_json
            .map(|json: String| serde_json::from_str(&json))
            .transpose()?;

        anyhow::Ok(SeedInfoRow::new(
            row.provisioning_pk_id,
            pk_upload_form,
            key_pair,
            register_journalist_form,
            sentinel_id_pk_upload_form,
            sentinel_id_key_pair,
            register_sentinel_profile_form,
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
