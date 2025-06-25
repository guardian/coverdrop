use crate::{
    coverdrop_service::CoverDropService,
    development_image_source::BringUpImageSource,
    docker::ImageAndTag,
    local_or_pvc_path::{LocalOrPvcPath, PvcPath},
};
use clap::{Parser, Subcommand};
use common::clap::AwsConfig;
use common::clap::Stage;
use std::{net::Ipv4Addr, num::NonZeroU8, path::PathBuf};

#[derive(Parser, Debug)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Command,
    #[clap(long)]
    pub skip_preflight_checks: bool,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Commands relating to the production cluster
    #[clap(alias = "prod")]
    Production {
        #[clap(subcommand)]
        command: ProductionCommand,
    },
    #[clap()]
    Staging {
        #[clap(subcommand)]
        command: StagingCommand,
    },
    /// Commands relating to development
    #[clap(alias = "dev")]
    Development {
        #[clap(subcommand)]
        command: DevelopmentCommand,
    },
    /// Commands relating to the CoverNode
    #[clap(name = "covernode")]
    CoverNode {
        #[clap(subcommand)]
        command: CoverNodeCommand,
    },
    /// Commands relating to the Identity API
    IdentityApi {
        /// The specific command for the Identity API
        #[clap(subcommand)]
        command: IdentityApiCommand,
    },
    /// Create a backup file for the entire cluster
    Backup {
        #[clap(subcommand)]
        command: BackupCommand,
    },
    /// Copy a file to or from a CoverDrop PersistentVolumeClaim.
    ///
    /// This command will set the correct file user, group and permissions.
    ///
    /// Files must fit into memory on the local device.
    ///
    /// Examples:
    ///   # Copy a local file to the CoverNode PVC.
    ///   coverup cp ./foo/bar covernode-persistentvolumeclaim:/baz/qux
    ///
    ///   # Copy a file from the CoverNode PVC to the
    ///   coverup cp covernode-persistentvolumeclaim:/baz/qux ./foo/bar
    ///
    ///  # Use an an absolute path to copy a local file which has a colon in the path
    ///  coverup cp /foo/bar:baz covernode-persistentvolumeclaim:/qux
    #[clap(alias = "cp")]
    CopyFile {
        /// Source path
        source: LocalOrPvcPath,
        /// Destination path
        destination: LocalOrPvcPath,
        /// Ignore safety checks that prevent overwriting existing files
        #[clap(long)]
        force: bool,
        /// Optionally provide a stage if you have multiple stages configured on your local machine
        /// If none is provided coverup will use the default kube context.
        #[clap(long)]
        stage: Option<Stage>,
    },
    /// List files in a persistent volume using a persistent volume claim and path
    ///
    /// Examples:
    ///   # List the contents of the CoverNode /foo/bar directory
    ///   coverup ls covernode-persistentvolumeclaim:/foo/bar
    #[clap(alias = "ls")]
    ListFiles {
        /// The persistent volume claim and path
        path: PvcPath,
        /// Display additional information
        #[clap(long, short)]
        long: bool,
        /// Optionally provide a stage if you have multiple stages configured on your local machine
        /// If none is provided coverup will use the default kube context.
        #[clap(long)]
        stage: Option<Stage>,
    },
    /// Open a shell on a data copier pod connected to a service's PVC
    DataCopierShell {
        /// The service you wish to open the shell against
        service: CoverDropService,
        /// Optionally provide a stage if you have multiple stages configured on your local machine
        /// If none is provided coverup will use the default kube context.
        #[clap(long)]
        stage: Option<Stage>,
    },
    JournalistVault {
        #[clap(subcommand)]
        command: JournalistVaultCommand,
    },
    /// Verify various objects
    Verify {
        #[clap(subcommand)]
        command: VerifyCommand,
    },
    /// Administration commands for managing aspects of the protocol
    Admin {
        #[clap(subcommand)]
        command: AdminCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum ProductionCommand {
    /// Bring up a production cluster where you have one or more
    /// nodes already reachable on the network.
    BringUp {
        /// The IPv4 addresses of your nodes.
        #[clap(long, value_delimiter = ',', num_args = 1..)]
        nodes: Vec<Ipv4Addr>,
    },
    /// Opens a tunnel to the admin machine and sets up port forwarding for ArgoCD dashboard
    /// access. Note: the ssh user needs to exist on the admin machine and have
    /// a kubernetes config file at ~/.kube/config
    Argo {
        #[clap(long)]
        admin_machine_ip: Ipv4Addr,
        #[clap(long)]
        ssh_user: String,
        #[clap(long, default_value = "8086")]
        port: u16,
    },
    K8s {
        #[clap(long)]
        admin_machine_ip: Ipv4Addr,
        #[clap(long)]
        ssh_user: String,
        #[clap(long, default_value = "8445")]
        port: u16,
    },
    Longhorn {
        #[clap(long)]
        admin_machine_ip: Ipv4Addr,
        #[clap(long)]
        ssh_user: String,
        #[clap(long, default_value = "8444")]
        local_port: u16,
    },
}

#[derive(Debug, Subcommand)]
pub enum StagingCommand {
    /// Bring up a production cluster where you have one or more
    /// nodes already reachable on the network.
    TearDown {
        #[command(flatten)]
        aws_config: AwsConfig,
    },
    /// Create a tunnel to the staging k3s cluster.
    KubectlTunnel {
        /// Local port to use
        #[clap(long, default_value = "16443")]
        port: u16,
        #[command(flatten)]
        aws_config: AwsConfig,
    },
    /// Create a tunnel to the staging k3s cluster then setup port forwarding for the Argo dashboard.
    Argo {
        #[command(flatten)]
        aws_config: AwsConfig,
        /// Local port to use
        #[clap(long, default_value = "8085")]
        port: u16,
    },
    /// Create a tunnel to the staging k3s cluster then setup port forwarding for the longhorn dashboard.
    Longhorn {
        #[command(flatten)]
        aws_config: AwsConfig,
        /// Local port to use
        #[clap(long, default_value = "8444")]
        port: u16,
    },
    Minio {
        #[command(flatten)]
        aws_config: AwsConfig,
        /// Local port to use
        #[clap(long, default_value = "9001")]
        port: u16,
    },
    /// Create a tunnel to the staging k3s cluster then setup port forwarding for the kubernetes dashboard.
    K8s {
        #[command(flatten)]
        aws_config: AwsConfig,
        /// Local port to use
        #[clap(long, default_value = "8444")]
        port: u16,
    },
    /// Fetch the contents of ~/.kube/config from the remote k3s cluster using ssm and save it to ~/.coverup
    Kubeconfig {
        #[command(flatten)]
        aws_config: AwsConfig,
        /// Bucket to use to hold ssm output.
        #[clap(long, default_value = "coverdrop-ssm-output-staging")]
        ssm_output_bucket: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum DevelopmentCommand {
    /// Bring up a development cluster using multipass on your local
    /// development laptop
    BringUp {
        /// Where should coverup get it's images from.
        #[clap(long, value_enum, default_value_t = BringUpImageSource::Repository)]
        image_source: BringUpImageSource,
        /// The number of nodes to use in your development cluster
        #[clap(long, default_value = "3")]
        node_count: NonZeroU8,
        /// The number of CPUs per multipass VM
        #[clap(long, default_value = "3")]
        cpus_per_node: NonZeroU8,
        /// The amount of RAM per multipass node in gigabytes
        #[clap(long, default_value = "4")]
        ram_gb_per_node: NonZeroU8,
        /// The amount of storage per multipass node in gigabytes
        #[clap(long, default_value = "30")]
        storage_gb_per_node: NonZeroU8,
    },
    /// Build a service using the a local copy of cargo and copy it into a container.
    Build {
        /// The service you wish to build
        service: CoverDropService,
    },
    /// Watch the workspace root for changes and rebuild/upload images
    /// to the local development cluster.
    Watch,
    /// Copy multipass dev cluster kubeconfig to ~/.coverup/kubeconfig-DEV
    Kubeconfig {
        /// Location of SSH key
        #[clap(long, default_value = "~/.coverup/coverup-multipass-ssh")]
        ssh_key_path: PathBuf,
        /// SSH user
        #[clap(long, default_value = "ubuntu")]
        ssh_user: String,
    },
    /// Copy an image built locally to multipass
    CopyImageToMultipass {
        /// The name of the docker image and it's tag. Pass this flag multiple times to copy
        /// multiple images in one invocation
        #[clap(long)]
        image: Vec<ImageAndTag>,
        /// Optionally, copy across all images required for local development
        #[clap(long, default_value_t = false, conflicts_with = "image")]
        all: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum CoverNodeCommand {
    /// Get the healthcheck JSON from the CoverNode
    Healthcheck {
        #[clap(long)]
        stage: Option<Stage>,
    },
    /// Get the public components of the CoverNode's key pairs.
    ///
    /// This is useful when comparing the keys found in the public
    /// API against the CoverNode's own view of it's keys.
    ///
    /// For now there is only ever one CoverNode, in the future
    /// you will need to provide a CoverNode ID.
    PublicKeys {
        #[clap(long)]
        stage: Option<Stage>,
    },
}

#[derive(Debug, Subcommand)]
pub enum IdentityApiCommand {
    /// Get the healthcheck JSON from the Identity API
    Healthcheck {
        #[clap(long)]
        stage: Option<Stage>,
    },
    PublicKeys {
        #[clap(long)]
        stage: Option<Stage>,
    },
}

#[derive(Debug, Subcommand)]
pub enum BackupCommand {
    Create {
        #[clap(long, default_value = ".")]
        /// The directory that will contain the backup
        output_directory: PathBuf,
        /// Optionally provide a stage if you have multiple stages configured on your local machine
        /// If none is provided coverup will use the default kube context.
        #[clap(long)]
        stage: Option<Stage>,
    },
}

#[derive(Debug, Subcommand)]
pub enum JournalistVaultCommand {
    /// Derive an argon2 key given a journalist vault and password, then
    /// decrypt the vault and keep a sqlcipher session open.
    OpenVault {
        #[clap(long)]
        vault_path: PathBuf,
        #[clap(long)]
        password: Option<String>,
        #[clap(long)]
        password_path: Option<PathBuf>,
    },
    /// Derive an argon2 key given a journalist vault and password.
    /// This key can be used to open the vault with PRAGMA key="key";
    DeriveKey {
        #[clap(long)]
        vault_path: PathBuf,
        #[clap(long)]
        password: Option<String>,
        #[clap(long)]
        password_path: Option<PathBuf>,
    },
    /// Add a new trust anchor to an existing vault
    AddTrustAnchor {
        #[clap(long)]
        vault_path: PathBuf,
        #[clap(long)]
        password: Option<String>,
        #[clap(long, conflicts_with = "password")]
        password_path: Option<PathBuf>,
        #[clap(long)]
        trust_anchor_path: PathBuf,
    },
    // TODO: delete https://github.com/guardian/coverdrop-internal/issues/3100
    MigrateHexArgon2Database {
        #[clap(long)]
        vault_path: PathBuf,
        #[clap(long)]
        password: Option<String>,
        #[clap(long, conflicts_with = "password")]
        password_path: Option<PathBuf>,
    },
    AddProvisioningPublicKey {
        #[clap(long)]
        vault_path: PathBuf,
        #[clap(long)]
        password: Option<String>,
        #[clap(long, conflicts_with = "password")]
        password_path: Option<PathBuf>,
        #[clap(long)]
        journalist_provisioning_pk_path: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
pub enum VerifyCommand {
    JournalistProvisioningKeyPair {
        #[clap(long)]
        journalist_provisioning_key_pair_path: PathBuf,
        #[clap(long)]
        organization_public_key_path: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
pub enum AdminCommand {
    /// Generate an encrypted identity-api database
    ///
    /// Should be ran offline since it will have access
    /// to the provisioning key pairs.
    GenerateIdentityApiDatabase {
        /// Path to the directory containing at least one trust anchor,
        /// journalist and CoverNode provisioning key pairs.
        #[clap(long)]
        keys_path: PathBuf,
        /// Optionally provide a specific path to the new database
        #[clap(long, default_value = "./identity-api.db")]
        db_path: PathBuf,
    },
}
