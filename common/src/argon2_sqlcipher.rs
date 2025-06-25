use std::{path::Path, str::FromStr as _};

use argon2::password_hash::SaltString;
use reqwest::Url;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteSynchronous},
    ConnectOptions as _, Row as _, SqliteConnection, SqlitePool,
};
use tokio::io::AsyncReadExt as _;

use crate::crypto::{
    pbkdf::{derive_secret_box_key_with_configuration, derive_vault_key, Argon2Configuration},
    SecretBoxKey,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ConnectMode {
    CreateIfMissing,
    ErrorIfMissing,
}

// Private enum to pass key values to pragma from both legacy and argon2
enum KeyPragmaValue<'a> {
    Password(&'a str),
    Argon2Key(SecretBoxKey),
    // TODO delete https://github.com/guardian/coverdrop-internal/issues/3100
    HexArgon2Key(SecretBoxKey),
}

impl KeyPragmaValue<'_> {
    fn to_pragma_string(&self) -> String {
        match self {
            KeyPragmaValue::Password(password) => format!("'{}'", password),
            KeyPragmaValue::Argon2Key(key) => {
                format!("\"x'{}'\"", hex::encode_upper(key))
            }
            KeyPragmaValue::HexArgon2Key(key) => {
                format!("\"x'{}'\"", hex::encode(hex::encode(key)))
            }
        }
    }
}

// https://www.zetetic.net/sqlcipher/sqlcipher-api/#cipher_salt
const SQLCIPHER_SALT_LEN: usize = 16;

pub struct Argon2SqlCipher {
    pool: SqlitePool,
}

impl Argon2SqlCipher {
    fn sqlcipher_connection_options(
        path: impl AsRef<Path>,
        key_pragma_value: KeyPragmaValue,
        connect_mode: ConnectMode,
    ) -> anyhow::Result<SqliteConnectOptions> {
        let Some(path) = path.as_ref().to_str() else {
            anyhow::bail!("Path to database is not valid unicode");
        };

        let url = format!("sqlite://{}", path);
        let url = Url::from_str(&url)?;

        //
        // A note on journaling modes:
        // Our databases run in a variety of environments. Sometimes against network attached storage
        // such as NFS. As such, we stick with the default journaling mode for now.
        //
        Ok(SqliteConnectOptions::from_url(&url)?
            .disable_statement_logging()
            .synchronous(SqliteSynchronous::Full)
            .pragma("key", key_pragma_value.to_pragma_string())
            .create_if_missing(connect_mode == ConnectMode::CreateIfMissing))
    }

    pub async fn new(path: impl AsRef<Path>, password: &str) -> anyhow::Result<Self> {
        // First create a legacy database - we do this so that SQLCipher manages
        // the creation of the salt.
        {
            let pool = Self::new_legacy(&path, password).await?;

            let mut conn = pool.acquire().await?;

            Self::ensure_database_created(&mut conn).await?;
            Self::check_is_unlocked(&mut conn).await?;
            Self::rekey_database(&mut conn, password).await?;
        }

        // Now that we have the database and it's rekeyed with Argon2 we can
        // open it again using our `open_argon2` function.
        let (pool, _) = Self::open_argon2(&path, password).await?;

        Ok(Self { pool })
    }

    // TODO: delete https://github.com/guardian/coverdrop-internal/issues/3100
    pub async fn migrate_hex_argon2(path: impl AsRef<Path>, password: &str) -> anyhow::Result<()> {
        let path = path.as_ref();

        let (hex_argon2_pool, _) = Self::open_hex_argon2(&path, password).await?;

        tracing::warn!(
            "Successfully opened hex Argon2 database {}, attempting to upgrade to Argon2 KDF",
            path.display()
        );

        let mut conn = hex_argon2_pool.acquire().await?;
        Self::rekey_database(&mut conn, password).await?;

        tracing::info!("Successfully rekeyed database {}", path.display());

        drop(hex_argon2_pool);

        tracing::info!(
            "Attempting to open rekeyed database using Argon2 KDF {}",
            path.display()
        );

        if Self::open_argon2(&path, password).await.is_ok() {
            tracing::info!("Successfuly reopened rekeyed database");

            Ok(())
        } else {
            anyhow::bail!("Failed to reopen database with rekeyed Argon2 KDF")
        }
    }

    /// Open a database that already exists using the legacy keying mode (PBKDF2) and migrate it to
    /// use Argon2. If the database is already using Argon2 it will open it as normal.
    ///
    /// This function will be modified once all production databases have been migrated to work
    /// exclusively on argon2 based databases.
    pub async fn open_and_maybe_migrate_from_legacy(
        path: impl AsRef<Path>,
        password: &str,
    ) -> anyhow::Result<Self> {
        let path = path.as_ref();

        tracing::debug!(
            "Attempting to open SQLCipher database using Argon2 KDF: {}",
            path.display()
        );

        if let Ok((argon2_pool, _)) = Self::open_argon2(&path, password).await {
            return Ok(Self { pool: argon2_pool });
        }

        if let Ok(legacy_pool) = Self::open_legacy(&path, password).await {
            tracing::warn!(
                "Found legacy database {}, attempting to upgrade to Argon2 KDF",
                path.display()
            );

            let mut conn = legacy_pool.acquire().await?;
            Self::rekey_database(&mut conn, password).await?;

            tracing::info!("Successfully rekeyed database {}", path.display());

            drop(legacy_pool);

            tracing::info!(
                "Attempting to open rekeyed database using Argon2 KDF {}",
                path.display()
            );

            if let Ok((argon2_pool, _)) = Self::open_argon2(&path, password).await {
                return Ok(Self { pool: argon2_pool });
            }
        }

        anyhow::bail!(
            "Failed to open database {} with the given password using both legacy and Argon2 KDFs",
            path.display()
        );
    }

    pub async fn derive_database_key(
        path: impl AsRef<Path>,
        password: &str,
    ) -> anyhow::Result<String> {
        let (_, key) = Self::open_argon2(path.as_ref(), password).await?;

        Ok(hex::encode(key))
    }

    /// Once the database is unlocked we can convert it to a raw `SqlitePool`
    /// to avoid creating extra indirections.
    pub fn into_sqlite_pool(self) -> SqlitePool {
        self.pool
    }

    //
    // Internal utility functions
    //

    async fn new_legacy(path: impl AsRef<Path>, password: &str) -> anyhow::Result<SqlitePool> {
        let options = Self::sqlcipher_connection_options(
            path.as_ref(),
            KeyPragmaValue::Password(password),
            ConnectMode::CreateIfMissing,
        )?;

        let pool = SqlitePool::connect_with(options).await?;

        Ok(pool)
    }

    async fn open_legacy(path: impl AsRef<Path>, password: &str) -> anyhow::Result<SqlitePool> {
        let options = Self::sqlcipher_connection_options(
            path.as_ref(),
            KeyPragmaValue::Password(password),
            ConnectMode::ErrorIfMissing,
        )?;

        let pool = SqlitePool::connect_with(options).await?;

        let mut conn = pool.acquire().await?;
        Self::check_is_unlocked(&mut conn).await?;

        Ok(pool)
    }

    // TODO: delete https://github.com/guardian/coverdrop-internal/issues/3100
    async fn open_hex_argon2(
        path: impl AsRef<Path>,
        password: &str,
    ) -> anyhow::Result<(SqlitePool, SecretBoxKey)> {
        let salt_bytes = Self::salt_from_file(path.as_ref()).await?;

        let salt = SaltString::encode_b64(&salt_bytes).unwrap();
        let key =
            derive_secret_box_key_with_configuration(password, &salt, Argon2Configuration::V0)?;

        let options = Self::sqlcipher_connection_options(
            path,
            KeyPragmaValue::HexArgon2Key(key),
            ConnectMode::ErrorIfMissing,
        )?;

        let pool = SqlitePool::connect_with(options).await?;

        let mut conn = pool.acquire().await?;
        Self::check_is_unlocked(&mut conn).await?;

        Ok((pool, key))
    }

    /// Attempt to open the database using a Argon2 derived key, returns a SQLite pool and
    /// the key which successfully opened it
    async fn open_argon2(
        path: impl AsRef<Path>,
        password: &str,
    ) -> anyhow::Result<(SqlitePool, SecretBoxKey)> {
        let salt_bytes = Self::salt_from_file(path.as_ref()).await?;
        let key = derive_vault_key(password, salt_bytes)?;

        let options = Self::sqlcipher_connection_options(
            path,
            KeyPragmaValue::Argon2Key(key),
            ConnectMode::ErrorIfMissing,
        )?;

        let pool = SqlitePool::connect_with(options).await?;

        let mut conn = pool.acquire().await?;
        Self::check_is_unlocked(&mut conn).await?;

        Ok((pool, key))
    }

    async fn salt_from_pragma(
        conn: &mut SqliteConnection,
    ) -> anyhow::Result<[u8; SQLCIPHER_SALT_LEN]> {
        let row = sqlx::query("PRAGMA cipher_salt")
            .fetch_one(&mut *conn)
            .await?;

        let salt_hex: String = row.get(0);

        let mut salt_bytes = [0u8; SQLCIPHER_SALT_LEN];
        hex::decode_to_slice(salt_hex, &mut salt_bytes)?;
        Ok(salt_bytes)
    }

    async fn salt_from_file(path: impl AsRef<Path>) -> anyhow::Result<[u8; SQLCIPHER_SALT_LEN]> {
        let mut file = tokio::fs::File::open(path).await?;
        let mut salt_bytes = [0u8; SQLCIPHER_SALT_LEN];
        file.read_exact(&mut salt_bytes).await?;
        Ok(salt_bytes)
    }

    pub async fn rekey_database(
        conn: &mut SqliteConnection,
        new_password: &str,
    ) -> anyhow::Result<()> {
        let salt_bytes = Self::salt_from_pragma(&mut *conn).await?;
        let key = derive_vault_key(new_password, salt_bytes)?;
        let key = KeyPragmaValue::Argon2Key(key);

        sqlx::query(&format!("PRAGMA rekey = {};", key.to_pragma_string()))
            .execute(&mut *conn)
            .await?;

        Ok(())
    }

    async fn ensure_database_created(conn: &mut SqliteConnection) -> anyhow::Result<()> {
        // We use the VACUUM command to make SQLite actually write the database to disk
        sqlx::query("VACUUM").execute(conn).await?;
        Ok(())
    }

    async fn check_is_unlocked(conn: &mut SqliteConnection) -> anyhow::Result<()> {
        sqlx::query("SELECT name FROM sqlite_master")
            .execute(conn)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{NamedTempFile, TempDir};

    #[tokio::test]
    async fn test_create_and_open_argon2() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test.db");

        {
            assert!(!path.exists());
            Argon2SqlCipher::new(&path, "password").await.unwrap();
            assert!(path.exists());
        }

        {
            // Opens with correct password
            Argon2SqlCipher::open_and_maybe_migrate_from_legacy(&path, "password")
                .await
                .unwrap();
        }

        {
            // Fails with wrong password
            let result =
                Argon2SqlCipher::open_and_maybe_migrate_from_legacy(&path, "wrong password").await;
            assert!(result.is_err());
        }
    }

    #[tokio::test]
    async fn salt_is_different_for_different_dbs() {
        let file1 = NamedTempFile::new().unwrap();
        let path1 = file1.path().to_path_buf();

        let file2 = NamedTempFile::new().unwrap();
        let path2 = file2.path().to_path_buf();

        let db1 = Argon2SqlCipher::new(&path1, "password").await.unwrap();
        let db2 = Argon2SqlCipher::new(&path2, "password").await.unwrap();

        let pool1 = db1.into_sqlite_pool();
        let mut conn1 = pool1.acquire().await.unwrap();

        let pool2 = db2.into_sqlite_pool();
        let mut conn2 = pool2.acquire().await.unwrap();

        let file_salt1 = Argon2SqlCipher::salt_from_file(&path1).await.unwrap();
        let file_salt2 = Argon2SqlCipher::salt_from_file(&path2).await.unwrap();

        let pragma_salt1 = Argon2SqlCipher::salt_from_pragma(&mut conn1).await.unwrap();
        let pragma_salt2 = Argon2SqlCipher::salt_from_pragma(&mut conn2).await.unwrap();

        assert_ne!(file_salt1, file_salt2);
        assert_ne!(pragma_salt1, pragma_salt2);

        assert_eq!(file_salt1, pragma_salt1);
        assert_eq!(file_salt2, pragma_salt2);
    }

    #[tokio::test]
    async fn migrate_legacy_to_argon2() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_path_buf();

        {
            // Use internal function to force the creation of a legacy database
            Argon2SqlCipher::new_legacy(&path, "password")
                .await
                .unwrap();
            assert!(path.exists());
        }

        {
            // Perform the open -> migrate
            Argon2SqlCipher::open_and_maybe_migrate_from_legacy(&path, "password")
                .await
                .unwrap();
        }

        {
            // Try to open with legacy approach using internal function, should fail
            let legacy_open_result = Argon2SqlCipher::open_legacy(&path, "password").await;
            assert!(legacy_open_result.is_err());
        }

        {
            // Opening with internal argon2 function, should succeed
            let argon2_open_result = Argon2SqlCipher::open_argon2(&path, "password").await;
            assert!(argon2_open_result.is_ok());
        }
    }
}
