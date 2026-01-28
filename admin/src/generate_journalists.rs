use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use common::{
    api::{
        forms::{PostJournalistForm, PostJournalistIdPublicKeyForm},
        models::journalist_id::JournalistIdentity,
    },
    client::JournalistStatus,
    crypto::keys::role::Role,
    protocol::{
        self,
        keys::{
            load_anchor_org_pks, load_journalist_provisioning_key_pairs_with_parent_org_pks,
            AnchorOrganizationPublicKey,
        },
        roles::JournalistProvisioning,
    },
    Error,
};
use itertools::Itertools;
use journalist_vault::{JournalistVault, ReplacementStrategy, PASSWORD_EXTENSION, VAULT_EXTENSION};
use tokio::fs;

pub struct JournalistVaultPaths {
    pub vault_path: PathBuf,
    pub password_path: PathBuf,
}

#[allow(clippy::too_many_arguments)]
pub async fn generate_journalist(
    keys_path: impl AsRef<Path>,
    display_name: String,
    id: Option<String>,
    sort_name: Option<String>,
    description: String,
    is_desk: bool,
    password: &str,
    status: JournalistStatus,
    vault_path: impl AsRef<Path>,
    now: DateTime<Utc>,
    trust_anchors: Vec<AnchorOrganizationPublicKey>,
) -> anyhow::Result<JournalistVaultPaths> {
    let org_pks = load_anchor_org_pks(&keys_path, now)?;

    // TODO use load_journalist_provisioning_key_pairs once we no longer need to pass org_pks to JournalistVault::create
    // https://github.com/guardian/coverdrop-internal/issues/3788
    let org_pks_and_journalist_provisioning_key_pairs =
        load_journalist_provisioning_key_pairs_with_parent_org_pks(&keys_path, &org_pks, now)?;

    let latest_journalist_provisioning_key_pair = org_pks_and_journalist_provisioning_key_pairs
        .iter()
        .map(|(_, journalist_provisioning_key_pair)| journalist_provisioning_key_pair)
        .max_by_key(|key_pair| key_pair.public_key().not_valid_after)
        .cloned()
        .ok_or_else(|| Error::LatestKeyPairNotFound(JournalistProvisioning::display()))?;

    let org_and_journalist_provisioning_pks = org_pks_and_journalist_provisioning_key_pairs
        .into_iter()
        .map(|(org_pk, journalist_provisioning_key_pair)| {
            (org_pk, journalist_provisioning_key_pair.to_public_key())
        })
        .collect::<Vec<_>>();

    let journalist_id = id.unwrap_or_else(|| display_name.to_lowercase().replace(' ', "_"));
    let journalist_id = JournalistIdentity::new(&journalist_id)?;

    let sort_name = sort_name_from_display_name(sort_name, &display_name)?;

    //
    // Create vault
    //

    // If the user provides a directory then stick /{journalist_id}.vault
    // on the end. Otherwise use the full path as provided.
    let mut vault_path = if vault_path.as_ref().is_dir() {
        vault_path.as_ref().join(journalist_id.as_ref())
    } else {
        vault_path.as_ref().into()
    };

    vault_path.set_extension(VAULT_EXTENSION);

    let vault = JournalistVault::create(
        &vault_path,
        password,
        &journalist_id,
        &org_and_journalist_provisioning_pks,
        now,
        trust_anchors,
    )
    .await?;

    //
    // Create seed forms for upload once there's an internet connection
    //

    let journalist_id_key_pair = protocol::keys::generate_journalist_id_key_pair(
        &latest_journalist_provisioning_key_pair,
        now,
    );

    let register_journalist_form = PostJournalistForm::new(
        journalist_id.clone(),
        display_name,
        sort_name,
        description,
        is_desk,
        status,
        &latest_journalist_provisioning_key_pair,
        now,
    )?;

    let pk_upload_form = PostJournalistIdPublicKeyForm::new(
        journalist_id,
        journalist_id_key_pair.public_key().to_untrusted(),
        false,
        &latest_journalist_provisioning_key_pair,
        now,
    )?;

    vault
        .add_vault_setup_bundle(
            latest_journalist_provisioning_key_pair.public_key(),
            journalist_id_key_pair,
            pk_upload_form,
            Some(register_journalist_form),
            ReplacementStrategy::Keep,
        )
        .await?;

    let password_path = vault_path.with_extension(PASSWORD_EXTENSION);

    fs::write(&password_path, password).await?;

    Ok(JournalistVaultPaths {
        vault_path,
        password_path,
    })
}

/// If no sort name is provided we can make an attempt at generating one from the display name
///
/// We don't want to make too many assumptions about names so we only want to attempt this if
/// the name is all ASCII and has only two words in it.
///
/// This allows "Joe Bloggs" to become "bloggs joe"
/// Names such as "Guido van Rossum" and "이세돌" will require a sort name to be manually provided.
fn sort_name_from_display_name(
    sort_name: Option<String>,
    display_name: &str,
) -> anyhow::Result<String> {
    sort_name.map(Ok).unwrap_or_else(|| {
        if !display_name.is_ascii() {
            anyhow::bail!(
                "Non-ascii characters in provided display name. Please manually provide a sort name using the --sort-name flag."
            );
        }

        let lower_name = display_name
            .to_lowercase();

        let split_display_name = lower_name.split(' ')
            .collect::<Vec<&str>>();

        if split_display_name.len() > 2 {
            anyhow::bail!(
                "More than two words in given display name. Please manually provide a sort name using the --sort-name flag."
            );
        } else {
            let rev_iter = split_display_name.into_iter().rev();
            Ok(Itertools::intersperse(rev_iter, " ").collect())
        }
    })
}
