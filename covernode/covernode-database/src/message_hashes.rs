use chrono::{DateTime, Utc};
use common::protocol::constants::JOURNALIST_MSG_KEY_VALID_DURATION;
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::sqlite::SqliteTypeInfo;
use sqlx::{Database, Sqlite};

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct MessageHash([u8; 32]);

impl MessageHash {
    pub fn new(hash: [u8; 32]) -> Self {
        Self(hash)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl TryFrom<&[u8]> for MessageHash {
    type Error = anyhow::Error;

    fn try_from(bytes: &[u8]) -> anyhow::Result<Self> {
        let hash: [u8; 32] = bytes.try_into()?;
        Ok(Self(hash))
    }
}

impl sqlx::Type<Sqlite> for MessageHash {
    fn type_info() -> SqliteTypeInfo {
        <Vec<u8> as sqlx::Type<Sqlite>>::type_info()
    }

    fn compatible(ty: &SqliteTypeInfo) -> bool {
        <Vec<u8> as sqlx::Type<Sqlite>>::compatible(ty)
    }
}

impl<'q> sqlx::Encode<'q, Sqlite> for MessageHash {
    fn encode_by_ref(
        &self,
        buf: &mut <Sqlite as Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, BoxDynError> {
        <Vec<u8> as sqlx::Encode<'q, Sqlite>>::encode(self.0.to_vec(), buf)
    }
}

impl<'r> sqlx::Decode<'r, Sqlite> for MessageHash {
    fn decode(value: <Sqlite as Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        let bytes = <&[u8] as sqlx::Decode<Sqlite>>::decode(value)?;
        MessageHash::try_from(bytes).map_err(Into::into)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, sqlx::Type)]
#[sqlx(transparent)]
pub struct MessageHashExpiry(DateTime<Utc>);

impl MessageHashExpiry {
    /// Takes an expiry time and adds the journalist message key duration
    /// to ensure the hash will expire after the inner message or outer message.
    pub fn new(expires_at: DateTime<Utc>) -> Self {
        Self(expires_at + JOURNALIST_MSG_KEY_VALID_DURATION)
    }

    pub fn into_inner(self) -> DateTime<Utc> {
        self.0
    }

    pub fn is_expired_at(&self, now: DateTime<Utc>) -> bool {
        self.0 <= now
    }
}

pub type MessageHashesWithExpiries = Vec<(MessageHash, MessageHashExpiry)>;
