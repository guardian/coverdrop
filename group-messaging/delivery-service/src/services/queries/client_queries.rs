use common::api::models::journalist_id::JournalistIdentity;
use common::time;
use delivery_service_lib::PROTOCOL_VERSION;
use openmls::prelude::tls_codec::Serialize as TlsSerialize;
use openmls::prelude::KeyPackageIn;
use openmls_rust_crypto::RustCrypto;
use sqlx::{PgPool, Postgres, Transaction};

use delivery_service_lib::tls_serialized::TlsSerialized;

#[derive(Clone)]
pub struct ClientQueries {
    pool: PgPool,
}

impl ClientQueries {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Register a new client with the given ID and initial key packages
    /// Each key package should be paired with its hash
    pub async fn register_client(
        &self,
        client_id: &JournalistIdentity,
        key_packages: Vec<KeyPackageIn>,
    ) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        sqlx::query!(
            "INSERT INTO clients (client_id, created_at) VALUES ($1, $2)",
            client_id,
            time::now()
        )
        .execute(&mut *tx)
        .await?;

        Self::insert_key_packages_in_tx(&mut tx, client_id, key_packages).await?;

        tx.commit().await?;

        Ok(())
    }

    /// Check if a client exists
    pub async fn client_exists(&self, client_id: &JournalistIdentity) -> anyhow::Result<bool> {
        let mut connection = self.pool.acquire().await?;

        let result = sqlx::query!(
            "SELECT client_id FROM clients WHERE client_id = $1",
            client_id
        )
        .fetch_optional(&mut *connection)
        .await?;

        Ok(result.is_some())
    }

    /// Insert key packages for an existing client
    /// Each key package should be paired with its hash
    pub async fn insert_key_packages(
        &self,
        client_id: &JournalistIdentity,
        key_packages: Vec<KeyPackageIn>,
    ) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;
        Self::insert_key_packages_in_tx(&mut tx, client_id, key_packages).await?;
        tx.commit().await?;
        Ok(())
    }

    async fn insert_key_packages_in_tx(
        tx: &mut Transaction<'_, Postgres>,
        client_id: &JournalistIdentity,
        key_packages: Vec<KeyPackageIn>,
    ) -> anyhow::Result<()> {
        let published_at = time::now();
        let crypto = &RustCrypto::default();

        for key_package in key_packages {
            let key_package_bytes = key_package.tls_serialize_detached()?;

            // Validate the key package before inserting
            let key_package = key_package.validate(crypto, PROTOCOL_VERSION)?;

            // Calculate the hash of the key package - this is used to uniquely identify it.
            let key_package_hash = key_package.hash_ref(crypto)?.as_slice().to_vec();

            sqlx::query!(
                r#"INSERT INTO key_packages (key_package_hash, client_id, published_at, key_package)
                VALUES ($1, $2, $3, $4)"#,
                key_package_hash,
                client_id,
                published_at,
                key_package_bytes,
            )
            .execute(&mut **tx)
            .await?;
        }

        Ok(())
    }

    /// Get all client IDs
    pub async fn get_all_client_ids(&self) -> anyhow::Result<Vec<JournalistIdentity>> {
        let mut connection = self.pool.acquire().await?;

        let rows = sqlx::query!(
            r#"
                SELECT client_id AS "client_id: JournalistIdentity" FROM clients
            "#
        )
        .fetch_all(&mut *connection)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| row.client_id)
            .collect::<Vec<_>>())
    }

    /// Consume a key package for the given client ID, returning one if available
    pub async fn consume_key_package(
        &self,
        client_id: &JournalistIdentity,
    ) -> anyhow::Result<Option<KeyPackageIn>> {
        let mut tx = self.pool.begin().await?;

        // Lock and select the oldest unconsumed key package
        // SKIP LOCKED ensures concurrent requests get different packages
        let row = sqlx::query!(
            r#"
                SELECT
                    key_package_hash,
                    key_package AS "key_package: TlsSerialized"
                FROM key_packages
                WHERE client_id = $1
                AND consumed_at IS NULL
                ORDER BY published_at ASC
                LIMIT 1
                FOR UPDATE SKIP LOCKED
            "#,
            client_id
        )
        .fetch_optional(&mut *tx)
        .await?;

        if let Some(record) = row {
            // Mark it as consumed
            sqlx::query!(
                "UPDATE key_packages SET consumed_at = $1 WHERE key_package_hash = $2",
                time::now(),
                record.key_package_hash
            )
            .execute(&mut *tx)
            .await?;

            tx.commit().await?;

            let key_package = record.key_package.deserialize::<KeyPackageIn>()?;
            Ok(Some(key_package))
        } else {
            tx.rollback().await?;
            Ok(None)
        }
    }
}
