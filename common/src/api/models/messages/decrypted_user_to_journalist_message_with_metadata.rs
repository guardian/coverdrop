use chrono::{DateTime, Utc};

use crate::api::models::dead_drops::DeadDropId;
use crate::protocol::constants::MESSAGE_PADDING_LEN;
use crate::protocol::keys::UserPublicKey;
use crate::{Error, FixedSizeAuthenticatedMessageText, FixedSizeMessageText};

/// Once the migration of authenticated messages is fully rolled out,
/// we should be able to remove LegacyUnauthenticated messages and therefore [UserToJournalistMessagePayload].
// TODO: Tracked in https://github.com/guardian/coverdrop-internal/issues/4240
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UserToJournalistMessagePayload {
    Authenticated(FixedSizeAuthenticatedMessageText),
    LegacyUnauthenticated(FixedSizeMessageText),
}

impl UserToJournalistMessagePayload {
    pub fn from_vec_checked(bytes: Vec<u8>) -> Result<Self, Error> {
        if bytes.len() == MESSAGE_PADDING_LEN as usize {
            Ok(UserToJournalistMessagePayload::LegacyUnauthenticated(
                FixedSizeMessageText::from_vec_checked(bytes)?,
            ))
        } else {
            Ok(UserToJournalistMessagePayload::Authenticated(
                FixedSizeAuthenticatedMessageText::from_vec_checked(bytes)?,
            ))
        }
    }

    pub fn from_vec_unchecked(bytes: Vec<u8>) -> Self {
        if bytes.len() == MESSAGE_PADDING_LEN as usize {
            UserToJournalistMessagePayload::LegacyUnauthenticated(
                FixedSizeMessageText::from_vec_unchecked(bytes),
            )
        } else {
            UserToJournalistMessagePayload::Authenticated(
                FixedSizeAuthenticatedMessageText::from_vec_unchecked(bytes),
            )
        }
    }

    pub fn to_string(&self) -> Result<String, Error> {
        match self {
            UserToJournalistMessagePayload::Authenticated(pcs) => pcs.to_string(),
            UserToJournalistMessagePayload::LegacyUnauthenticated(pcs) => pcs.to_string(),
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        match self {
            UserToJournalistMessagePayload::Authenticated(pcs) => pcs.as_bytes(),
            UserToJournalistMessagePayload::LegacyUnauthenticated(pcs) => pcs.as_bytes(),
        }
    }

    pub fn check_is_valid_string(&self) -> bool {
        match self {
            UserToJournalistMessagePayload::Authenticated(pcs) => pcs.check_is_valid_string(),
            UserToJournalistMessagePayload::LegacyUnauthenticated(pcs) => {
                pcs.check_is_valid_string()
            }
        }
    }
}

/// Wrapper for [UserToJournalistMessage] including the ID and created_at timestamp
/// of the U2J dead drop which contained it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecryptedUserToJournalistMessageWithMetadata {
    pub reply_key: UserPublicKey,
    pub payload: UserToJournalistMessagePayload,
    pub unsigned_dead_drop_id: DeadDropId,
    pub dead_drop_created_at: DateTime<Utc>,
}
