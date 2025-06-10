//! Various functions for Password Based Key Derivation Function (PBKDF) operations,
//! aka turning user provided, low entropy, passwords into cryptographic keys.

use crate::Error;
use argon2::{
    password_hash::{Salt, SaltString},
    Argon2,
};
use chacha20poly1305::aead::OsRng;

use super::secret_box::{SecretBoxKey, KEY_LEN};

/// Default passphrase length for Argon2 when using the EFF word list.
/// See: `docs/client_passphrase_configurations.md`
pub const DEFAULT_PASSPHRASE_WORDS: usize = 5;

#[derive(Debug, Clone, Copy)]
pub enum Argon2Configuration {
    /// The original variant which we now consider insecure
    V0,

    /// The more secure variant
    V1,
}

impl Argon2Configuration {
    pub fn params(&self) -> Result<argon2::Params, argon2::Error> {
        match self {
            // default parameters from the argon2 crate as per version 0.4.1
            Argon2Configuration::V0 => argon2::ParamsBuilder::new()
                .m_cost(4096)
                .t_cost(3)
                .p_cost(1)
                .build(),

            // Inspired by OPSLIMIT_SENSITIVE and MEMLIMIT_SENSITIVE from libsodium
            Argon2Configuration::V1 => argon2::ParamsBuilder::new()
                .m_cost(100 * 1024)
                .t_cost(6)
                .p_cost(4)
                .build(),
        }
    }
}

pub fn generate_salt() -> SaltString {
    SaltString::generate(&mut OsRng)
}

pub fn derive_secret_box_key(password: &str, salt: &SaltString) -> anyhow::Result<SecretBoxKey> {
    derive_secret_box_key_with_configuration(password, salt, Argon2Configuration::V0)
}

pub fn derive_secret_box_key_with_configuration(
    password: &str,
    salt: &SaltString,
    configuration: Argon2Configuration,
) -> anyhow::Result<SecretBoxKey> {
    let params = configuration
        .params()
        .map_err(|_| Error::Argon2BadParameters)?;
    let argon2 = Argon2::new(
        argon2::Algorithm::default(),
        argon2::Version::default(),
        params,
    );

    let salt = Salt::try_from(salt.as_ref())
        .map_err(|_| anyhow::anyhow!("Failed to create salt from salt string"))?;
    let mut salt_arr = [0u8; KEY_LEN];
    let salt_bytes = salt.decode_b64(&mut salt_arr).unwrap();

    let mut key: [u8; KEY_LEN] = [0; KEY_LEN];
    argon2
        .hash_password_into(password.as_bytes(), salt_bytes, &mut key)
        .unwrap();

    Ok(SecretBoxKey::from(key))
}

pub fn derive_vault_key(password: &str, salt_bytes: [u8; 16]) -> anyhow::Result<SecretBoxKey> {
    let salt = SaltString::encode_b64(&salt_bytes).unwrap();
    derive_secret_box_key_with_configuration(password, &salt, Argon2Configuration::V1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use argon2::password_hash::SaltString;

    #[test]
    fn can_derive_key_fixed_salt() -> anyhow::Result<()> {
        let password = "password";
        let salt = SaltString::from_b64("Y2hpcCBzcGljZQ").unwrap();

        let key = derive_secret_box_key(password, &salt)?;

        assert_eq!(
            key,
            [
                68, 61, 37, 212, 143, 246, 132, 85, 219, 215, 21, 61, 94, 84, 97, 122, 127, 53, 22,
                204, 168, 98, 43, 239, 98, 163, 182, 161, 217, 182, 61, 155
            ]
            .into()
        );

        Ok(())
    }

    #[test]
    fn will_fail_if_password_is_wrong() -> anyhow::Result<()> {
        let salt = SaltString::from_b64("Y2hpcCBzcGljZQ").unwrap();

        let password_1 = "password";
        let key_1 = derive_secret_box_key(password_1, &salt)?;

        let password_2 = "a different password";
        let key_2 = derive_secret_box_key(password_2, &salt)?;

        assert_ne!(key_1, key_2);

        Ok(())
    }

    #[test]
    fn will_fail_if_salt_is_different() -> anyhow::Result<()> {
        let password = "password";

        let salt_1 = SaltString::from_b64("Y2hpcCBzcGljZQ").unwrap();
        let salt_2 = SaltString::from_b64("ZGlmZmVyZW50").unwrap();

        let key_1 = derive_secret_box_key(password, &salt_1)?;
        let key_2 = derive_secret_box_key(password, &salt_2)?;

        assert_ne!(key_1, key_2);

        Ok(())
    }

    #[test]
    fn measure_time_for_hashing_for_variants() -> anyhow::Result<()> {
        let password = "password";
        let salt = SaltString::generate(&mut rand::thread_rng());

        for configuration in [Argon2Configuration::V0, Argon2Configuration::V1] {
            let start = std::time::Instant::now();
            let _key =
                derive_secret_box_key_with_configuration(password, &salt, configuration).unwrap();
            let duration = start.elapsed();

            println!("Hashing with {:?} took {:?}", configuration, duration);
        }

        Ok(())
    }
}
