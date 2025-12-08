/// The port used by Postgres within the Docker network.
/// On the host the port will be randomized and can be retrieved using
/// `container.get_host_port_ipv4(PORT)`
pub const POSTGRES_PORT: u16 = 5432;

/// The Postgres user used in the test container
pub const POSTGRES_USER: &str = "coverdrop";

/// The Postgres password used in the test container
pub const POSTGRES_PASSWORD: &str = "coverdrop";

/// The Postgres database used in the test container
pub const POSTGRES_DB: &str = "coverdrop";

/// The port used by Kinesis within the Docker network.
/// On the host the port will be randomized and can be retrieved using
/// `container.get_host_port_ipv4(PORT)`
pub const KINESIS_PORT: u16 = 4567;

// The port used by the api in the test containers
pub const API_PORT: u16 = api::DEFAULT_PORT;

// The port used by varnish in the test containers
pub const VARNISH_PORT: u16 = 80;

// The port used by the identity API in the test containers
pub const IDENTITY_API_PORT: u16 = identity_api::DEFAULT_PORT;

// The port used by the U2J Appender service in the test containers
pub const U2J_APPENDER_PORT: u16 = u2j_appender::DEFAULT_PORT;

// the password for the covernode sqlite db
pub const COVERNODE_DB_PASSWORD: &str = "covernode-db-password-secret";

// the password for the identity-api sqlite db
pub const IDENTITY_API_DB_PASSWORD: &str = "identity-api-db-password-secret";

// default minio port
pub const MINIO_PORT: u16 = 9000;
