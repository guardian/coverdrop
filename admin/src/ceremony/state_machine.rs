use chrono::{DateTime, Utc};

use common::{
    api::{forms::PostCoverNodeIdPublicKeyForm, models::covernode_id::CoverNodeIdentity},
    backup::keys::{
        generate_backup_id_key_pair, generate_backup_msg_key_pair, BackupIdKeyPair,
        BackupMsgKeyPair,
    },
    crypto::keys::{
        serde::StorableKeyMaterial,
        signing::{SignedPublicSigningKey, SigningKeyPair},
    },
    protocol::{
        keys::{
            generate_covernode_id_key_pair, generate_covernode_provisioning_key_pair,
            generate_journalist_provisioning_key_pair, generate_organization_key_pair,
            load_org_key_pairs, CoverNodeProvisioningKeyPair, JournalistProvisioningKeyPair,
        },
        roles::Organization,
    },
    system::keys::{generate_admin_key_pair, AdminKeyPair},
};
use covernode_database::Database;
use std::{fmt::Display, num::NonZeroU8, path::PathBuf, vec};
use strum::EnumIter;

use super::*;

/// The copy of the organization key pair being handled
#[derive(Clone)]
pub(super) enum Copy {
    Primary,
    Secondary,
}

impl Display for Copy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Copy::Primary => write!(f, "1"),
            Copy::Secondary => write!(f, "2"),
        }
    }
}

pub(super) enum UsbDevice {
    OrgKeyPair(Copy),
    OrgKeyPairPassword(Copy),
    PublicKeysAndForms,
    EditorialStaff,
    IdentityAPI,
    CoverNodeDatabase,
    AdminKeyPair,
    BackupKeys,
}

impl UsbDevice {
    pub(super) fn description(&self) -> &str {
        match self {
            UsbDevice::OrgKeyPair(Copy::Primary) => "Org key pair copy 1",
            UsbDevice::OrgKeyPairPassword(Copy::Primary) => "Org key pair password 1",
            UsbDevice::OrgKeyPair(Copy::Secondary) => "Org key pair copy 2",
            UsbDevice::OrgKeyPairPassword(Copy::Secondary) => "Org key pair password 2",
            UsbDevice::PublicKeysAndForms => "Public keys and forms",
            UsbDevice::EditorialStaff => "Editorial staff",
            UsbDevice::IdentityAPI => "Identity API",
            UsbDevice::CoverNodeDatabase => "CoverNode database",
            UsbDevice::AdminKeyPair => "Admin keys",
            UsbDevice::BackupKeys => "Backup keys",
        }
    }

    fn all_devices() -> Vec<Self> {
        vec![
            UsbDevice::OrgKeyPair(Copy::Primary),
            UsbDevice::OrgKeyPairPassword(Copy::Primary),
            UsbDevice::OrgKeyPair(Copy::Secondary),
            UsbDevice::OrgKeyPairPassword(Copy::Secondary),
            UsbDevice::PublicKeysAndForms,
            UsbDevice::EditorialStaff,
            UsbDevice::IdentityAPI,
            UsbDevice::CoverNodeDatabase,
            UsbDevice::AdminKeyPair,
            UsbDevice::BackupKeys,
        ]
    }

    // certain usb devices required only for initial setup
    fn required_for_ceremony_type(&self, ceremony_type: &CeremonyType) -> bool {
        match self {
            UsbDevice::CoverNodeDatabase => *ceremony_type == CeremonyType::InitialSetup,
            _ => true,
        }
    }
}

#[derive(Clone)]
pub struct CeremonyState {
    pub ceremony_type: CeremonyType,
    pub output_directory: PathBuf,
    pub covernode_count: Option<NonZeroU8>,
    pub assume_yes: AssumeYes,
    pub covernode_db_password: Option<String>,
    pub now: DateTime<Utc>,

    // Key pairs
    pub org_key_pair: Option<SigningKeyPair<Organization, SignedPublicSigningKey<Organization>>>,
    pub journalist_provisioning_key_pair: Option<JournalistProvisioningKeyPair>,
    pub covernode_provisioning_key_pair: Option<CoverNodeProvisioningKeyPair>,
    pub admin_key_pair: Option<AdminKeyPair>,
    pub backup_id_key_pair: Option<BackupIdKeyPair>,
    pub backup_msg_key_pair: Option<BackupMsgKeyPair>,

    // Key pair files
    pub encrypted_journalist_provisioning_key_pair_file: Option<PathBuf>,
    pub journalist_provisioning_key_pair_file: Option<PathBuf>,
    pub encrypted_covernode_provisioning_key_pair_file: Option<PathBuf>,
    pub admin_key_pair_file: Option<PathBuf>,
    pub backup_id_key_pair_file: Option<PathBuf>,
    pub backup_msg_key_pair_file: Option<PathBuf>,
    // Anchor organization public key file
    pub anchor_org_pk_file: Option<PathBuf>,
    // Bundles
    pub set_system_status_available_bundle: Option<PathBuf>,
    pub public_key_forms_bundle: Option<PathBuf>,
}

impl CeremonyState {
    pub fn new(
        ceremony_type: CeremonyType,
        assume_yes: AssumeYes,
        output_directory: impl AsRef<Path>,
        covernode_count: Option<NonZeroU8>,
        covernode_db_password: Option<String>,
        org_key_pair_path: Option<PathBuf>,
        now: DateTime<Utc>,
    ) -> Self {
        let mut org_key_pair = None;

        // if this is a setup ceremony, covernode args are required
        if ceremony_type == CeremonyType::InitialSetup
            && (covernode_count.is_none() || covernode_db_password.is_none())
        {
            panic!(
                "covernode_count and covernode_db_password are required for initial setup ceremony"
            );
        }

        if let Some(org_key_pair_path) = org_key_pair_path {
            let org_key_pairs =
                load_org_key_pairs(org_key_pair_path, now).expect("read org key pairs");

            if org_key_pairs.len() != 1 {
                panic!("Must have exactly one org key pair in directory")
            }

            org_key_pair = org_key_pairs.into_iter().next();
        }

        Self {
            ceremony_type,
            output_directory: output_directory.as_ref().to_owned(),
            covernode_count,
            assume_yes,
            covernode_db_password,
            now,
            // Key pairs
            org_key_pair,
            journalist_provisioning_key_pair: None,
            covernode_provisioning_key_pair: None,
            admin_key_pair: None,
            backup_id_key_pair: None,
            backup_msg_key_pair: None,
            // Files
            encrypted_journalist_provisioning_key_pair_file: None,
            journalist_provisioning_key_pair_file: None,
            encrypted_covernode_provisioning_key_pair_file: None,
            admin_key_pair_file: None,
            backup_id_key_pair_file: None,
            backup_msg_key_pair_file: None,
            set_system_status_available_bundle: None,
            anchor_org_pk_file: None,
            public_key_forms_bundle: None,
        }
    }
}

#[derive(Clone, EnumIter)]
pub enum CeremonyStep {
    /// The initial step prompting the user to start the ceremony
    InitialStep,
    GenerateOrganizationKeyPair,
    GenerateJournalistProvisioningKeyPair,
    GenerateCoverNodeProvisioningKeyPair,
    GenerateCoverNodeDatabase,
    GenerateAdminKeyPair,
    GenerateBackupKeys,
    GenerateSystemStatusBundle,
    GenerateAnchorOrganizationPublicKeyFile,
    GeneratePublicKeyFormsBundle,
    /// Final step prompting the user to delete files
    FinalStep,
}

impl Display for CeremonyStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CeremonyStep::InitialStep => write!(f, "Initial Step"),
            CeremonyStep::GenerateOrganizationKeyPair => {
                write!(f, "Generate Organization Key Pair")
            }
            CeremonyStep::GenerateJournalistProvisioningKeyPair => {
                write!(f, "Generate Journalist Provisioning Key Pair")
            }
            CeremonyStep::GenerateCoverNodeProvisioningKeyPair => {
                write!(f, "Generate CoverNode Provisioning Key Pair")
            }
            CeremonyStep::GenerateCoverNodeDatabase => {
                write!(f, "Generate CoverNode Database")
            }
            CeremonyStep::GenerateAdminKeyPair => {
                write!(f, "Generate Admin Key Pair")
            }
            CeremonyStep::GenerateBackupKeys => {
                write!(f, "Generate Backup Keys")
            }
            CeremonyStep::GenerateSystemStatusBundle => {
                write!(f, "Generate System Status Bundle")
            }
            CeremonyStep::GenerateAnchorOrganizationPublicKeyFile => {
                write!(f, "Generate Anchor Organization Public Key File")
            }
            CeremonyStep::GeneratePublicKeyFormsBundle => {
                write!(f, "Generate Public Key Forms Bundle")
            }
            CeremonyStep::FinalStep => write!(f, "Final Step"),
        }
    }
}

impl CeremonyStep {
    /// Execute the current step
    pub async fn execute(&self, state: &mut CeremonyState) -> anyhow::Result<()> {
        let CeremonyState {
            output_directory,
            covernode_count,
            assume_yes,
            covernode_db_password,
            now,
            ..
        } = state;

        if *assume_yes == AssumeYes::AlwaysAsk {
            println!("\n=== {self} ===");
        }

        match self {
            CeremonyStep::InitialStep => {
                let mut confirm_environment_message = match &state.ceremony_type {
                    CeremonyType::InitialSetup => String::from(
                        "You're about to start the initial key ceremony to set up CoverDrop.",
                    ),
                    CeremonyType::OrgKeyRotation => String::from(
                        "You're about to start the organization key rotation ceremony.",
                    ),
                };
                confirm_environment_message.push_str(
                    "\nThis ceremony should be run on a tails machine with no network access, and with a clock set to the correct time.",
                );
                confirm_environment_message
                    .push_str("\nPlease confirm you are running in such an environment.");

                // always make the user confirm that they're in a safe environment, regardless of assume_yes
                // Need to skip this in tests.
                #[cfg(not(test))]
                ask_user_to_confirm(&confirm_environment_message, AssumeYes::AlwaysAsk)?;

                let mut required_materials_message = String::from(
                    "You will need the following removable devices to store the material.",
                );
                required_materials_message
                    .push_str("\nMake sure all devices are appropriately labelled as");

                for device in UsbDevice::all_devices() {
                    if device.required_for_ceremony_type(&state.ceremony_type) {
                        required_materials_message
                            .push_str(&format!("\n- {}", device.description()));
                    }
                }
                required_materials_message.push_str("\nDo you have all removable devices to hand?");
                ask_user_to_confirm(&required_materials_message, *assume_yes)?;
                Ok(())
            }
            CeremonyStep::GenerateOrganizationKeyPair => {
                let (org_key_pair, org_key_pair_preprovided) = match state.org_key_pair {
                    Some(ref org_key_pair) => (org_key_pair.clone(), true),
                    None => {
                        let prompt = [
                            "The organization key pair is about to be created.",
                            "This is the most sensitive key pair and needs to be stored as securely as possible.",
                            "If the secret key is ever leaked, the whole system's integrity is compromised and a new ceremony will have to be started.",
                            "Do you understand?"
                        ].join("\n");

                        ask_user_to_confirm(&prompt, *assume_yes)?;
                        let org_key_pair = generate_organization_key_pair(*now);
                        state.org_key_pair = Some(org_key_pair.clone());
                        (org_key_pair, false)
                    }
                };

                if org_key_pair_preprovided {
                    println!("Using pre-provided organization key pair.");
                } else {
                    println!("Generated new organization key pair.");
                }
                age_encrypt_org_key_pair(
                    Copy::Primary,
                    &org_key_pair,
                    output_directory,
                    assume_yes,
                )?;
                age_encrypt_org_key_pair(
                    Copy::Secondary,
                    &org_key_pair,
                    output_directory,
                    assume_yes,
                )?;

                Ok(())
            }
            CeremonyStep::GenerateJournalistProvisioningKeyPair => {
                let journalist_provisioning_key_pair = generate_journalist_provisioning_key_pair(
                    state.org_key_pair.as_ref().unwrap(),
                    *now,
                );
                state.journalist_provisioning_key_pair =
                    Some(journalist_provisioning_key_pair.clone());

                let untrusted_key_pair = journalist_provisioning_key_pair.to_untrusted();

                let encrypted_key_pair_file_path =
                    age_encrypt_and_prompt_transfer_of_provisioning_key(
                        &untrusted_key_pair,
                        output_directory,
                        &UsbDevice::IdentityAPI,
                        assume_yes,
                    )?;
                state.encrypted_journalist_provisioning_key_pair_file =
                    Some(encrypted_key_pair_file_path);

                // Only write the plaintext copy to disk after the encrypted handoff above is confirmed.
                let plaintext_file_path = untrusted_key_pair.save_to_disk(output_directory)?;
                state.journalist_provisioning_key_pair_file = Some(plaintext_file_path.clone());

                let prompt = [
                    &format!("A plaintext copy of the journalist provisioning key pair has been created at the following location: \n'{}'",
                        plaintext_file_path.to_str().unwrap()
                    ),
                    &format!("Copy the journalist provisioning key pair to removable device '{}'. This will be used by editorial staff to create journalist profiles and vaults.", UsbDevice::EditorialStaff.description()),
                    "Have you completed this step?",
                ]
                .join("\n");

                ask_user_to_confirm(&prompt, *assume_yes)?;
                Ok(())
            }
            CeremonyStep::GenerateCoverNodeProvisioningKeyPair => {
                let covernode_provisioning_key_pair = generate_covernode_provisioning_key_pair(
                    state.org_key_pair.as_ref().unwrap(),
                    *now,
                );
                state.covernode_provisioning_key_pair =
                    Some(covernode_provisioning_key_pair.clone());

                let file_path = age_encrypt_and_prompt_transfer_of_provisioning_key(
                    &covernode_provisioning_key_pair.to_untrusted(),
                    output_directory,
                    &UsbDevice::IdentityAPI,
                    assume_yes,
                )?;
                state.encrypted_covernode_provisioning_key_pair_file = Some(file_path);

                Ok(())
            }
            CeremonyStep::GenerateCoverNodeDatabase => {
                if state.ceremony_type == CeremonyType::OrgKeyRotation {
                    // Skip CoverNode DB generation during org key rotation
                    println!(
                        "Skipping CoverNode database generation during organization key rotation."
                    );
                    return Ok(());
                }

                for covernode_id in 1..=(*covernode_count).unwrap().into() {
                    let covernode_identity = CoverNodeIdentity::from_node_id(covernode_id);

                    let db_path = output_directory.join(format!("{covernode_identity}.db"));
                    let db =
                        Database::open(&db_path, covernode_db_password.as_ref().unwrap()).await?;

                    let covernode_id_key_pair = generate_covernode_id_key_pair(
                        state.covernode_provisioning_key_pair.as_ref().unwrap(),
                        *now,
                        &covernode_identity,
                    );

                    let now = time::now();

                    let form = PostCoverNodeIdPublicKeyForm::new(
                        covernode_identity.clone(),
                        covernode_id_key_pair.public_key().to_untrusted(),
                        state.covernode_provisioning_key_pair.as_ref().unwrap(),
                        now,
                    )?;

                    db.insert_setup_bundle(&form, &covernode_id_key_pair, now)
                        .await?;
                }

                let prompt = [
                    "The CoverNode database(s) has been created.",
                    "Make sure the following step is completed:",
                    &format!(
                        "- Move the CoverNode database(s) to removable device '{}'.",
                        UsbDevice::CoverNodeDatabase.description()
                    ),
                    "This will be used by the CoverNode.",
                    "Have you completed this step?",
                ]
                .join("\n");

                ask_user_to_confirm(&prompt, *assume_yes)?;

                Ok(())
            }
            CeremonyStep::GenerateAdminKeyPair => {
                let admin_key_pair =
                    generate_admin_key_pair(state.org_key_pair.as_ref().unwrap(), *now);
                state.admin_key_pair = Some(admin_key_pair.clone());

                let file_path = admin_key_pair
                    .to_untrusted()
                    .save_to_disk(output_directory)?;
                state.admin_key_pair_file = Some(file_path);

                let prompt = [
                    "The admin key pair has been created.",
                    "Make sure the following step is completed:",
                    &format!(
                        "- Move the admin key pair to removable device '{}'.",
                        UsbDevice::AdminKeyPair.description()
                    ),
                    "Have you completed this step?",
                ]
                .join("\n");

                ask_user_to_confirm(&prompt, *assume_yes)?;
                Ok(())
            }
            CeremonyStep::GenerateBackupKeys => {
                let backup_id_key_pair =
                    generate_backup_id_key_pair(state.org_key_pair.as_ref().unwrap(), time::now());
                state.backup_id_key_pair = Some(backup_id_key_pair.clone());
                let backup_id_key_pair_path = backup_id_key_pair
                    .to_untrusted()
                    .save_to_disk(&output_directory)?;
                state.backup_id_key_pair_file = Some(backup_id_key_pair_path.clone());

                let backup_msg_key_pair =
                    generate_backup_msg_key_pair(&backup_id_key_pair, time::now());
                state.backup_msg_key_pair = Some(backup_msg_key_pair.clone());
                let backup_msg_key_pair_path = backup_msg_key_pair
                    .to_untrusted()
                    .save_to_disk(&output_directory)?;
                state.backup_msg_key_pair_file = Some(backup_msg_key_pair_path.clone());

                let prompt = [
                    &format!(
                        "The backup id key has been created and written to {}.",
                        backup_id_key_pair_path.to_str().unwrap()
                    ),
                    &format!(
                        "The backup msg key has been created and written to {}.",
                        backup_msg_key_pair_path.to_str().unwrap()
                    ),
                    "Make sure the following step is completed:",
                    &format!(
                        "- Move both of the backup keys to removable device '{}'.",
                        UsbDevice::BackupKeys.description()
                    ),
                    "Have you completed this step?",
                ]
                .join("\n");

                ask_user_to_confirm(&prompt, *assume_yes)?;

                Ok(())
            }
            CeremonyStep::GenerateSystemStatusBundle => {
                if state.ceremony_type == CeremonyType::OrgKeyRotation {
                    println!("Skipping set system status bundle during organization key rotation.");
                    return Ok(());
                }
                let bundle = save_set_system_status_available_bundle(
                    output_directory,
                    state.admin_key_pair.as_ref().unwrap(),
                )?;
                state.set_system_status_available_bundle = Some(bundle);

                let prompt = [
                    "The set system status bundle has been created.",
                    "This will be used in the post-ceremony to send a request to the API to mark CoverDrop as available.",
                    "Make sure the following step is completed:",
                    &format!(
                        "- Move the set system status bundle to removable device '{}'.",
                        UsbDevice::PublicKeysAndForms.description()
                    ),
                    "Have you completed this step?",
                ]
                .join("\n");

                ask_user_to_confirm(&prompt, *assume_yes)?;
                Ok(())
            }
            CeremonyStep::GenerateAnchorOrganizationPublicKeyFile => {
                let anchor_org_pk = state
                    .org_key_pair
                    .as_ref()
                    .unwrap()
                    .public_key()
                    .clone()
                    .into_anchor();

                let anchor_org_pk_file = anchor_org_pk
                    .to_untrusted()
                    .save_to_disk(output_directory)?;

                state.anchor_org_pk_file = Some(anchor_org_pk_file.clone());

                let prompt = [
                    &format!(
                        "The anchor organization public key file has been written to {}.",
                        anchor_org_pk_file.to_str().unwrap()
                    ),
                    "This trust anchor needs to be added to each component of the system.",
                    "Make sure the following step is completed:",
                    &format!(
                        "- Copy anchor organization public key file to removable device '{}'. This will be used to verify the journalist provisioning key pair.",
                        UsbDevice::EditorialStaff.description()
                    ),
                    &format!(
                        "- Copy anchor organization public key file to removable device '{}'.",
                        UsbDevice::PublicKeysAndForms.description()
                    ),
                    "Have you completed this step?"
                ].join("\n");

                ask_user_to_confirm(&prompt, *assume_yes)?;
                Ok(())
            }
            CeremonyStep::GeneratePublicKeyFormsBundle => {
                let bundle = save_public_key_forms_bundle(
                    output_directory,
                    state.org_key_pair.as_ref().unwrap(),
                    state
                        .journalist_provisioning_key_pair
                        .as_ref()
                        .unwrap()
                        .public_key()
                        .to_untrusted(),
                    state
                        .covernode_provisioning_key_pair
                        .as_ref()
                        .unwrap()
                        .public_key()
                        .to_untrusted(),
                    state
                        .admin_key_pair
                        .as_ref()
                        .unwrap()
                        .public_key()
                        .to_untrusted(),
                    state.backup_id_key_pair.as_ref().unwrap(),
                    state
                        .backup_msg_key_pair
                        .as_ref()
                        .unwrap()
                        .public_key()
                        .to_untrusted(),
                )?;
                state.public_key_forms_bundle = Some(bundle);

                let prompt = [
                    &format!(
                        "The public key forms bundle has been written to {}.",
                        state.public_key_forms_bundle.as_ref().unwrap().to_str().unwrap()
                    ),
                    "This will be used in the post-ceremony to bootstrap the public key infrastructure.",
                    "Make sure the following step is completed:",
                    &format!(
                        "- Move public key forms bundle to removable device '{}'.",
                        UsbDevice::PublicKeysAndForms.description()
                    ),
                    "Have you completed this step?",
                ]
                .join("\n");

                ask_user_to_confirm(&prompt, *assume_yes)?;
                Ok(())
            }
            CeremonyStep::FinalStep => {
                let prompt = [
                    "The key ceremony is complete.",
                    "All the key material and forms generated during the ceremony must now be deleted.",
                    "Make sure you have transferred the bundles to the appropriate devices.",
                    "Have you deleted all the bundles?"
                ].join("\n");

                ask_user_to_confirm(&prompt, *assume_yes)?;

                Ok(())
            }
        }
    }
}
