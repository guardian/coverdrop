use std::{num::NonZeroU8, path::PathBuf};

use clap::{Parser, Subcommand};
use common::api::models::{
    covernode_id::CoverNodeIdentity, general::SystemStatus, journalist_id::JournalistIdentity,
};
use common::aws::ssm::prefix::ParameterPrefix;
use common::clap::AwsConfig;
use common::client::JournalistStatus;
use reqwest::Url;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
#[allow(clippy::enum_variant_names)]
pub enum Commands {
    /// Must be ran offline!
    ///
    /// Run the CoverDrop set up ceremony. Creates various key pairs
    /// and a post-ceremony bundle to be uploaded to the API once
    /// the administrator is online
    RunSetupCeremony {
        /// The directory where the secret and public bundles will be saved
        #[clap(long)]
        output_directory: PathBuf,
        /// The number of CoverNode keys you wish to create. Must be between 1 to 256 inclusive.
        #[clap(long)]
        covernode_count: NonZeroU8,
        /// Automatic yes to prompts. Assume "yes" as answer to all prompts and run non-interactively.
        #[clap(short = 'y', long = "yes")]
        assume_yes: bool,
        /// The password for the CoverNode key database.
        #[clap(long)]
        covernode_db_password: String,
        /// Optionally, run the ceremony with a provided root organization key pair. This will
        /// cause the tool to skip the creation of a new organization key pair and use the provided
        /// one instead.
        #[clap(long)]
        org_key_pair_path: Option<PathBuf>,
    },
    /// After a the key creation ceremony has been performed, this command can be used to upload the organization
    /// public key, covernode and journalist provisioning public keys, and admin public key to the API.
    UploadKeysToApi {
        /// The path to the key bundle that was created during the offline proportion of the ceremony.
        #[clap(long)]
        bundle_directory_path: PathBuf,
        /// The address of the CoverDrop API server
        #[clap(long)]
        api_url: Url,

        #[command(flatten)]
        aws_config: AwsConfig,

        /// The SSM parameter prefix needed to fetch the trusted organization public key.
        #[clap(name = "aws-parameter-prefix", long, env = "AWS_PARAMETER_PREFIX")]
        parameter_prefix: Option<ParameterPrefix>,
    },
    /// Generate the top-level key pair which will be used to
    /// sign all the other keys.
    GenerateOrganizationKeyPair {
        /// The directory you wish to create the key files in
        #[clap(long)]
        keys_path: PathBuf,
    },
    GenerateJournalistProvisioningKeyPair {
        /// The directory you wish to create the key files in
        /// and the path to the directory containing the organization's
        /// public and secret keys
        #[clap(long)]
        keys_path: PathBuf,
        /// The address of the CoverDrop API server
        #[clap(long)]
        api_url: Url,
        /// Don't upload the key to the API, useful when generating keys for test purposes
        #[clap(long)]
        do_not_upload_to_api: bool,
    },
    #[clap(name = "generate-covernode-provisioning-key-pair")]
    GenerateCoverNodeProvisioningKeyPair {
        /// The directory you wish to create the key files in
        /// and the path to the directory containing the organization's
        /// public and secret keys
        #[clap(long)]
        keys_path: PathBuf,
        /// The address of the CoverDrop API server
        #[clap(long)]
        api_url: Url,
        /// Don't upload the key to the API, useful when generating keys for test purposes
        #[clap(long)]
        do_not_upload_to_api: bool,
    },
    #[clap(name = "generate-covernode-identity-key-pair")]
    GenerateCoverNodeIdentityKeyPair {
        /// The identity of the CoverNode. Must match the regex format `covernode_\d\d\d`
        #[clap(long)]
        covernode_id: CoverNodeIdentity,
        /// The directory you wish to create the key files in
        /// and the path to the directory containing the organization's
        /// public and secret keys
        #[clap(long)]
        keys_path: PathBuf,
        /// The address of the CoverDrop API server
        #[clap(long)]
        api_url: Url,
        /// Don't upload the key to the API, useful when generating keys for test purposes
        #[clap(long)]
        do_not_upload_to_api: bool,
    },
    #[clap(name = "generate-covernode-messaging-key-pair")]
    GenerateCoverNodeMessagingKeyPair {
        /// The directory you wish to create the key files in
        /// and the path to the directory containing the organization's
        /// public and secret keys
        #[clap(long)]
        keys_path: PathBuf,
        /// The address of the CoverDrop API server
        #[clap(long)]
        api_url: Url,
        /// Don't upload the key to the API, useful when generating keys for test purposes
        #[clap(long)]
        do_not_upload_to_api: bool,
    },
    #[cfg(feature = "integration-tests")]
    #[clap(name = "generate-journalist-messaging-keys-for-integration-test")]
    GenerateJournalistMessagingKeysForIntegrationTest {
        #[clap(long)]
        keys_path: PathBuf,
    },
    #[clap(name = "generate-covernode-database")]
    GenerateCoverNodeDatabase {
        /// The ID of the CoverNode
        #[clap(long)]
        covernode_id: CoverNodeIdentity,
        /// The directory you wish to create the key files in
        /// and the path to the directory containing the organization's
        /// public and secret keys
        #[clap(long)]
        keys_path: PathBuf,
        /// The password used to encrypt the database
        #[clap(long)]
        db_password: String,
        /// The directory where you wish to save the Covernode database
        #[clap(long)]
        output_path: PathBuf,
    },
    /// Generate keys for journalists or desks
    GenerateJournalist {
        /// The name this journalist or desk
        #[clap(long)]
        display_name: String,
        /// Optionally, override the identity of the journalist. Useful when the journalist has non-ascii characters in their name.
        #[clap(long)]
        id: Option<String>,
        /// A description for the journalist. If the journalist is a desk they can
        /// have a long description, if it's an individual reporter the description
        /// must be short.
        #[clap(long)]
        description: String,
        /// Optionally, force a specific password for the mailbox
        #[clap(long)]
        password: Option<String>,
        /// The initial status of the journalist. Possible values: VISIBLE, HIDDEN_FROM_UI, HIDDEN_FROM_RESPONSE
        #[clap(long, default_value = "VISIBLE")]
        status: JournalistStatus,
        /// The sort name for this journalist, if the display name is a simple forename
        /// and surname then this can be generated automatically
        #[clap(long)]
        sort_name: Option<String>,
        /// The path to where you wish to create the new mailbox, by default this is the current directory
        #[clap(long, default_value = "./")]
        vault_path: PathBuf,
        /// Is this journalist a desk?
        #[clap(long)]
        is_desk: bool,
        /// The keys directory containing the trust anchor and journalist provisioning key pair
        #[clap(long)]
        keys_path: PathBuf,
    },
    /// Updates the passphrase of a journalist vault to a new randomly generated one. Where applicable, the encryption
    /// scheme is updated to the latest variant. The new passphrase is saved in the respective .password file.
    ChangeVaultPassword {
        /// The path to the vault you wish to change the passphrase for
        #[clap(long)]
        vault_path: PathBuf,
        /// The current passphrase of the vault
        #[clap(long)]
        current_password: String,
    },
    UpdateJournalist {
        /// The ID of the journalist you wish to update
        #[clap(long)]
        journalist_id: JournalistIdentity,
        /// The address of the CoverDrop API server
        #[clap(long)]
        api_url: Url,
        /// The new display name of the journalist
        #[clap(long)]
        display_name: Option<String>,
        /// The new sort name of the journalist
        #[clap(long)]
        sort_name: Option<String>,
        /// The new desk status of the journalist
        #[clap(long)]
        is_desk: Option<bool>,
        /// The new description of the journalist
        #[clap(long)]
        description: Option<String>,
        /// The keys directory containing the trust anchor and journalist provisioning key pair
        #[clap(long)]
        keys_path: PathBuf,
    },
    ReseedJournalistVaultIdKeyPair {
        #[clap(long)]
        journalist_id: JournalistIdentity,
        /// The keys directory containing the journalist provisioning public and secret keys
        #[clap(long)]
        keys_path: PathBuf,
        #[clap(long)]
        vault_path: PathBuf,
        #[clap(long)]
        password: Option<String>,
        #[clap(long, conflicts_with = "password")]
        password_path: Option<PathBuf>,
    },
    /// Delete a given journalist
    DeleteJournalist {
        /// The address of the CoverDrop API server
        #[clap(long)]
        api_url: Url,
        /// The path of the delete journalist form you wish to submit to the API
        #[clap(long)]
        form_path: PathBuf,
    },
    /// Delete a given journalist
    DeleteJournalistForm {
        // The ID of the journalist you want to delete
        journalist_id: JournalistIdentity,
        /// The keys directory containing the journalist provisioning key pair and trust anchor
        #[clap(long)]
        keys_path: PathBuf,
        /// The path to output the delete journalist form
        #[clap(long)]
        output_path: PathBuf,
    },
    /// Generate test vectors for the currently compiled version. These are used for ensuring
    /// cross-platform and cross-version compatibility.
    GenerateTestVectors {
        /// The path where the test vectors are created. Defaults to common/tests/vectors.
        #[clap(default_value = "common/tests/vectors")]
        path: PathBuf,
    },
    /// Generates files for the mobile apps that contain the main constants from the common Rust
    /// files.
    GenerateMobileConstantsFiles {
        /// The path where the Android Kotlin file is created.
        #[clap(
            default_value = "android/core/src/main/java/com/theguardian/coverdrop/core/generated/Constants.kt"
        )]
        android_path: PathBuf,
        /// The path where the iOS Swift file is created.
        #[clap(
            default_value = "ios/reference/CoverDropCore/Sources/CoverDropCore/generated/Constants.swift"
        )]
        ios_path: PathBuf,
    },
    /// Create a new key pair for updating the system status.
    GenerateAdminKeyPair {
        /// The directory you wish to create the key files in
        /// and the path to the directory containing the organization's
        /// public and secret keys
        #[clap(long)]
        keys_path: PathBuf,
        /// The address of the CoverDrop API server
        #[clap(long)]
        api_url: Url,
        /// Don't upload the key to the API, useful when generating keys for test purposes
        #[clap(long)]
        do_not_upload_to_api: bool,
    },
    #[clap(name = "generate-backup-identity-key-pair")]
    GenerateBackupIdentityKeyPair {
        #[clap(long)]
        keys_path: PathBuf,
    },
    #[clap(name = "generate-backup-messaging-key-pair")]
    GenerateBackupMessagingKeyPair {
        #[clap(long)]
        keys_path: PathBuf,
    },
    /// Update system status. API consumers will be able to get the system
    /// status information using the GET endpoint /v1/status
    UpdateSystemStatus {
        /// The address of the CoverDrop API server
        #[clap(long)]
        api_url: Url,
        /// The path to the directory containing the system status key pair
        #[clap(long)]
        keys_path: PathBuf,
        #[clap(long, value_enum)]
        status: SystemStatus,
        /// Additional information regarding the status of the system
        #[clap(long)]
        description: String,
    },
    PostReloadLoggingForm {
        /// URL of the service
        #[clap(long)]
        service_url: Url,
        /// Path to the directory containing the admin key pair.
        #[clap(long)]
        keys_path: PathBuf,
        /// See more: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html
        #[clap(long)]
        rust_log_directive: String,
    },
    /// Outputs the digest of the organisation's public key in a human readable format. These can
    /// be exchanged out-of-band, e.g. by printing them, to verify the trust anchors in the app.
    /// The organisation keys are loaded either from the API, from the local file system, or both.
    PrintOrganisationKeyDigests {
        #[clap(long)]
        api_url: Option<Url>,
        /// Path to the directory containing the trusted organization public keys
        #[clap(long)]
        keys_path: Option<PathBuf>,
    },
    /// Prepares a backup init bundle that can be subsequently submitted using the
    /// `backup-initiate-restore-submit` step which will contact the API to retrieve
    /// the backup data for the specified journalist and the latest key hierarchy.
    /// This command is to be run on the air-gapped administrator machine.
    BackupInitiateRestorePrepare {
        /// Path to the directory containing the backup admin key pair
        #[clap(long)]
        keys_path: PathBuf,
        /// The ID of the journalist you wish to restore the backup for
        #[clap(long)]
        journalist_id: JournalistIdentity,
        /// Path to the directory where the bundle file for the subsequent restore submit step will
        /// be saved.
        #[clap(long)]
        bundle_path: PathBuf,
    },
    /// Submits the backup init bundle created using the `backup-initiate-restore-prepare` command
    /// to the API to retrieve the backup data and key hierarchy. The output is a bundle
    /// response that can be used in the subsequent `backup-initiate-restore-finalize` step.
    /// This command is to be run on any online machine.
    BackupInitiateRestoreSubmit {
        /// The path to the bundle file created using the `backup-initiate-restore-prepare` command
        #[clap(long)]
        bundle_path: PathBuf,
        /// The address of the CoverDrop API server
        #[clap(long)]
        api_url: Url,
        /// The path where the bundle response file for the subsequent restore finalize step will
        /// be saved.
        #[clap(long)]
        output_path: PathBuf,
    },
    /// Finalizes the backup restore process using the bundle response created
    /// using the `backup-initiate-restore-submit` command. This creates an intermediate
    /// backup file and encrypted recovery shares that can be used in the subsequent
    /// `backup-complete-restore` step.
    /// This command is to be run on the air-gapped administrator machine.
    BackupInitiateRestoreFinalize {
        /// The path to the bundle response file created using the
        /// `backup-initiate-restore-submit` command
        #[clap(long)]
        bundle_response_path: PathBuf,
        /// Path to the directory containing the backup admin key pair
        #[clap(long)]
        keys_path: PathBuf,
        /// The path where the intermediate backup file and encrypted shares for the subsequent
        /// complete restore step will be saved.
        #[clap(long)]
        output_path: PathBuf,
    },
    /// Completes the restore of a journalist vault using the intermediate backup file created
    /// using the `backup-initiate-restore` command and the encrypted recovery shares collected
    /// from the trusted contacts.
    /// This command is to be run on the air-gapped administrator machine.
    BackupCompleteRestore {
        /// The path to the intermediate backup file that was created using the
        /// `backup-initiate-restore` command
        #[clap(long)]
        in_progress_bundle_path: PathBuf,
        /// The path to the journalist vault you wish to restore the backup to
        #[clap(long)]
        restore_to_vault_path: PathBuf,
        /// Path to the directory containing the backup admin key pair
        #[clap(long)]
        keys_path: PathBuf,
        /// The encrypted recovery shares collected from the trusted contacts
        #[clap(long)]
        shares: Vec<String>,
    },
}
