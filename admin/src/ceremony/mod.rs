pub mod public_key_forms_bundle;
mod set_system_status_available_bundle;
mod state_machine;
mod tests;

use chrono::{DateTime, Utc};
use common::{
    api::api_client::ApiClient,
    aws::ssm::{
        client::SsmClient, parameters::ANCHOR_ORG_PK_SSM_PARAMETER, prefix::ParameterPrefix,
    },
    crypto::keys::untrusted::signing::UntrustedSignedPublicSigningKey,
    protocol::{
        keys::{load_anchor_org_pks, UntrustedAnchorOrganizationPublicKey},
        roles::AnchorOrganization,
    },
    throttle::Throttle,
    time,
};
use public_key_forms_bundle::*;
use serde::de::DeserializeOwned;
use state_machine::*;
use strum::IntoEnumIterator;

use self::set_system_status_available_bundle::{
    save_set_system_status_available_bundle, SetSystemStatusAvailableBundle,
};
use common::clap::AwsConfig;
use std::{
    collections::HashMap,
    fs::File,
    num::NonZeroU8,
    path::{Path, PathBuf},
    time::Duration,
};

const PUBLIC_KEY_FORMS_BUNDLE_FILENAME: &str = "public_key_forms.bundle.json";
const SET_SYSTEM_STATUS_AVAILABLE_BUNDLE_FILENAME: &str = "set_system_status_available.bundle.json";

/// Prompts the user to type in a confirmation word in order to proceed
/// with the next step of the ceremony
fn ask_user_to_confirm(message: &str, assume_yes: AssumeYes) -> anyhow::Result<()> {
    if assume_yes == AssumeYes::DefaultYes {
        return Ok(());
    }

    const CONFIRMATION_WORD: &str = "yes";

    loop {
        println!("{message}\nType '{CONFIRMATION_WORD}' to confirm.");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() == CONFIRMATION_WORD {
            break;
        }
    }

    Ok(())
}

#[derive(Clone, PartialEq)]
pub enum CeremonyType {
    InitialSetup,
    OrgKeyRotation,
}

#[derive(Clone, Copy, PartialEq)]
pub enum AssumeYes {
    AlwaysAsk,
    DefaultYes,
}

impl AssumeYes {
    pub fn from_boolean(assume_yes: bool) -> Self {
        if assume_yes {
            AssumeYes::DefaultYes
        } else {
            AssumeYes::AlwaysAsk
        }
    }
}

pub async fn run_key_ceremony(
    ceremony_type: CeremonyType,
    output_directory: impl AsRef<Path>,
    assume_yes: AssumeYes,
    covernode_count: Option<NonZeroU8>,
    covernode_db_password: Option<String>,
    org_key_pair_path: Option<PathBuf>,
    now: DateTime<Utc>,
) -> anyhow::Result<()> {
    if !output_directory.as_ref().is_dir() {
        anyhow::bail!("Provided path to output directory is not a directory");
    }

    let mut state = CeremonyState::new(
        ceremony_type,
        assume_yes,
        &output_directory,
        covernode_count,
        covernode_db_password,
        org_key_pair_path,
        now,
    );

    // Walk through every step of the ceremony until it's complete
    for step in CeremonyStep::iter() {
        step.execute(&mut state).await?;
    }

    Ok(())
}

/// Check that the bundles necessary for the post-ceremony actions exist on disk.
/// If they don't, an error message listing the missing files is displayed.
fn check_post_ceremony_bundles_exist(
    bundle_directory_path: impl AsRef<Path>,
    ceremony_type: CeremonyType,
) -> Result<(), anyhow::Error> {
    let base_path = bundle_directory_path.as_ref();

    let mut all_bundles = vec![base_path.join(PUBLIC_KEY_FORMS_BUNDLE_FILENAME)];

    // Only require system status bundle for initial setup
    if ceremony_type == CeremonyType::InitialSetup {
        all_bundles.push(base_path.join(SET_SYSTEM_STATUS_AVAILABLE_BUNDLE_FILENAME));
    }

    let all_bundles = all_bundles
        .iter()
        .map(|p| (p, p.exists()))
        .collect::<HashMap<_, _>>();

    let all_bundles_exist = all_bundles.iter().all(|(_, exists)| *exists);

    if !all_bundles_exist {
        let mut error_message = String::new();
        error_message.push_str(
            "The following files are required to run the post-ceremony actions, but were not found:\n",
        );
        for (path, _) in all_bundles.iter().filter(|&(_, exists)| !exists) {
            error_message.push_str(&path.to_string_lossy());
            error_message.push('\n');
        }
        anyhow::bail!("{}", error_message);
    }

    Ok(())
}

pub fn read_bundle_from_disk<T>(bundle_path: impl AsRef<Path>) -> anyhow::Result<T>
where
    T: DeserializeOwned,
{
    let reader = File::open(bundle_path)?;
    let bundle = serde_json::from_reader(reader)?;
    Ok(bundle)
}

pub async fn put_anchor_org_pk_parameter(
    ssm_client: &SsmClient,
    parameter_prefix: &ParameterPrefix,
    anchor_org_pk: &UntrustedSignedPublicSigningKey<AnchorOrganization>,
) -> anyhow::Result<()> {
    let anchor_org_pk = serde_json::to_string(&anchor_org_pk)?;

    ssm_client
        .put_string_parameter(
            parameter_prefix.get_parameter(ANCHOR_ORG_PK_SSM_PARAMETER),
            anchor_org_pk,
            "The organization's trusted public key",
        )
        .await?;

    Ok(())
}

pub async fn api_has_anchor_org_pk(
    api_client: &ApiClient,
    anchor_org_pk: &UntrustedAnchorOrganizationPublicKey,
) -> anyhow::Result<bool> {
    let has_key = api_client
        .get_public_keys()
        .await?
        .keys
        .org_pk_iter()
        // Converting to the key to a TOFU key allows us to not have to revalidated the entire hierarchy every time
        // this functionally does the same thing as validating but requires less cloning
        .any(|api_org_pk| api_org_pk.to_tofu_anchor() == *anchor_org_pk);

    Ok(has_key)
}

pub async fn upload_keys_to_api(
    bundle_path: impl AsRef<Path>,
    api_client: &ApiClient,
    aws_config: &AwsConfig,
    parameter_prefix: &Option<ParameterPrefix>,
    ceremony_type: CeremonyType,
) -> anyhow::Result<()> {
    // Make sure all files needed for post-ceremony actions exist
    check_post_ceremony_bundles_exist(&bundle_path, ceremony_type.clone())?;

    let base_path = bundle_path.as_ref();

    let anchor_org_pk = load_anchor_org_pks(base_path, time::now())?;

    // return an error if there isn't exactly one trusted org pk
    if anchor_org_pk.len() != 1 {
        anyhow::bail!(
            "Expected exactly one trusted organization public key, found {}",
            anchor_org_pk.len()
        );
    }
    let anchor_org_pk = &anchor_org_pk[0];
    let untrusted_anchor_org_pk = anchor_org_pk.to_untrusted();

    match &parameter_prefix {
        Some(parameter_prefix) => {
            let ssm_client = SsmClient::new(aws_config.region.to_owned(), aws_config.profile.to_owned()).await;
            put_anchor_org_pk_parameter(
                &ssm_client,
                parameter_prefix,
                &untrusted_anchor_org_pk,
            )
            .await?;
        },
        _ => println!("Running locally, not inserting trusted organization public key in the AWS parameter store")
    }

    let started_polling = time::now();
    let max_duration = chrono::Duration::minutes(10);
    let mut throttle = Throttle::new(Duration::from_secs(10));

    while !api_has_anchor_org_pk(api_client, &untrusted_anchor_org_pk).await? {
        let elapsed = time::now() - started_polling;

        println!(
            "Waiting for new organization key to appear in API (waited {}s/{}s)",
            elapsed.num_seconds(),
            max_duration.num_seconds()
        );

        if elapsed > max_duration {
            anyhow::bail!(
                "Trusted organization key does not appear in API after {} seconds of checking",
                elapsed.num_seconds()
            );
        }

        throttle.wait().await;
    }

    // Public keys bundle
    let bundle = base_path.join(PUBLIC_KEY_FORMS_BUNDLE_FILENAME);
    let bundle = read_bundle_from_disk::<PublicKeyFormsBundle>(bundle)?;

    api_client
        .post_journalist_provisioning_pk(bundle.journalist_provisioning_pk_form)
        .await?;

    api_client
        .post_covernode_provisioning_pk(bundle.covernode_provisioning_pk_form)
        .await?;

    api_client.post_admin_pk(bundle.admin_pk_form).await?;

    api_client
        .post_backup_signing_pk(bundle.backup_id_pk_form)
        .await?;

    api_client
        .post_backup_encryption_pk(bundle.backup_msg_pk_form)
        .await?;

    // Set system status available - only for initial setup
    if ceremony_type == CeremonyType::InitialSetup {
        let bundle = base_path.join(SET_SYSTEM_STATUS_AVAILABLE_BUNDLE_FILENAME);
        let bundle = read_bundle_from_disk::<SetSystemStatusAvailableBundle>(bundle)?;

        api_client
            .post_status_event_form(bundle.set_system_status_form)
            .await?;
    }

    Ok(())
}
