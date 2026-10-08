use std::io::{self, Seek, Write};

use crate::{Argon2Salt, Error};

const BASE_64_ENCODED_RECOMMENDED_SALT_LEN: usize = 22;

/// Unencrypted data for the mailbox - at the moment this is the same for the user and the journalist
#[derive(Clone)]
pub struct PlainMailboxData {
    pub salt: Argon2Salt,
}

impl PlainMailboxData {
    pub const SERIALIZED_LEN: usize = 1 // Length of argon2 salt
     + BASE_64_ENCODED_RECOMMENDED_SALT_LEN; // Argon2 salt

    /// Deserialize the plain mailbox data, the first byte is the length of the salt
    pub fn read(reader: &mut impl io::Read) -> anyhow::Result<Self> {
        let mut size_buf = [0; 1];
        reader.read_exact(&mut size_buf)?;

        let mut salt_buf = vec![0; size_buf[0] as usize];
        reader.read_exact(salt_buf.as_mut_slice())?;

        let salt = Argon2Salt::from_b64(std::str::from_utf8(&salt_buf)?)
            .map_err(|_| Error::Argon2SaltParse)?;

        Ok(PlainMailboxData { salt })
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

        let after = writer.stream_position()?;
        assert_eq!((after - before) as usize, Self::SERIALIZED_LEN);

        Ok(())
    }
}
