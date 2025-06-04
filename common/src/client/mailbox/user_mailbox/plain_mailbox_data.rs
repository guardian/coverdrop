use std::io::{self, Seek, Write};

use ed25519_dalek::SIGNATURE_LENGTH;

use crate::{
    client::mailbox::message_timestamp::MessageTimestamp,
    crypto::{keys::Ed25519PublicKey, Signature},
    protocol::{constants::ED25519_PUBLIC_KEY_LEN, keys::AnchorOrganizationPublicKey},
    Argon2Salt, Error,
};

const BASE_64_ENCODED_RECOMMENDED_SALT_LEN: usize = 22;
/// Unencrypted data for the mailbox - at the moment this is the same for the user and the journalist
#[derive(Clone)]
pub struct PlainMailboxData {
    pub salt: Argon2Salt,
    pub org_pks: Vec<AnchorOrganizationPublicKey>,
}

impl PlainMailboxData {
    pub const SERIALIZED_LEN: usize = 1 // Length of argon2 salt
     + BASE_64_ENCODED_RECOMMENDED_SALT_LEN // Argon2 salt
     + ED25519_PUBLIC_KEY_LEN // Trusted organization public key
     + SIGNATURE_LENGTH // Self signed signature
     + MessageTimestamp::SERIALIZED_LEN; // not valid after

    /// Deserialize the plain mailbox data, the first byte is the length of the salt
    pub fn read(reader: &mut impl io::Read) -> anyhow::Result<Self> {
        let mut size_buf = [0; 1];
        reader.read_exact(&mut size_buf)?;

        let mut salt_buf = vec![0; size_buf[0] as usize];
        reader.read_exact(salt_buf.as_mut_slice())?;

        let salt = Argon2Salt::from_b64(std::str::from_utf8(&salt_buf)?)
            .map_err(|_| Error::Argon2SaltParse)?;

        // TODO we should have a buffer of zero'd out org_pks so that we can support more than one

        let mut key_buf = [0; ED25519_PUBLIC_KEY_LEN];
        reader.read_exact(key_buf.as_mut_slice())?;
        let key = Ed25519PublicKey::from_bytes(&key_buf)?;

        let mut cert_buf = [0; SIGNATURE_LENGTH];
        reader.read_exact(cert_buf.as_mut_slice())?;
        let certificate = Signature::from_vec_unchecked(Vec::from(cert_buf));

        let MessageTimestamp(not_valid_after) = MessageTimestamp::read(reader)?;

        let org_pk = AnchorOrganizationPublicKey::new(key, certificate, not_valid_after);
        let org_pk = vec![org_pk];

        Ok(PlainMailboxData {
            salt,
            org_pks: org_pk,
        })
    }

    pub fn write<W>(&self, writer: &mut W) -> anyhow::Result<()>
    where
        W: Write + Seek,
    {
        let before = writer.stream_position()?;

        let salt_bytes = self.salt.as_str().as_bytes();
        assert_eq!(salt_bytes.len(), BASE_64_ENCODED_RECOMMENDED_SALT_LEN);

        let salt_len = u8::try_from(salt_bytes.len())?;

        writer.write_all(&[salt_len])?;
        writer.write_all(salt_bytes)?;

        // TODO we should have a buffer of zero'd out org_pks so that we can support more than one
        writer.write_all(self.org_pks[0].key.as_bytes())?;
        writer.write_all(&self.org_pks[0].certificate.to_bytes())?;
        let not_valid_after = MessageTimestamp(self.org_pks[0].not_valid_after);
        not_valid_after.write(writer)?;

        let after = writer.stream_position()?;
        assert_eq!((after - before) as usize, Self::SERIALIZED_LEN);

        Ok(())
    }
}
