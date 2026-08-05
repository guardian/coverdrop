use crate::protocol::keys::PublishedBackupIdPublicKeyFamilyList;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UntrustedBackupPublicKeyFamilyList(pub Vec<PublishedBackupIdPublicKeyFamilyList>);
