//! Public key families are a combination of an identity key and it's child
//! messaging keys. They are shared across

mod backup;
mod covernode;
mod identity_public_key_family;
mod identity_public_key_family_list;
mod journalist;
mod organization;
mod untrusted_identity_public_key_family;
mod untrusted_identity_public_key_family_list;

use crate::backup::roles::{BackupId, BackupMsg};
use crate::protocol::roles::{
    CoverNodeId, CoverNodeMessaging, CoverNodeProvisioning, JournalistId, JournalistMessaging,
    JournalistProvisioning, Organization,
};
pub use backup::*;
pub use covernode::*;
pub use identity_public_key_family::IdentityPublicKeyFamily;
pub use identity_public_key_family_list::IdentityPublicKeyFamilyList;
pub use journalist::*;
pub use organization::*;
pub use untrusted_identity_public_key_family::UntrustedIdentityPublicKeyFamily;
pub use untrusted_identity_public_key_family_list::UntrustedIdentityPublicKeyFamilyList;

// Type aliases for the top level key hierarchies
pub type CoverDropPublicKeyHierarchy = OrganizationPublicKeyFamilyList;
pub type PublishedCoverDropPublicKeyHierarchy = UntrustedOrganizationPublicKeyFamilyList;

// The ID -> Messaging key family is the same for both CoverNodes and Journalists so we use
// type aliases and a generic `IdentityPublicKeyFamilyList` for those.

// Covernode

// Published
pub type PublishedCoverNodeIdPublicKeyFamily =
    UntrustedIdentityPublicKeyFamily<CoverNodeProvisioning, CoverNodeId, CoverNodeMessaging>;
pub type PublishedCoverNodeIdPublicKeyFamilyList =
    UntrustedIdentityPublicKeyFamilyList<CoverNodeProvisioning, CoverNodeId, CoverNodeMessaging>;

// Verified
pub type CoverNodeIdPublicKeyFamily =
    IdentityPublicKeyFamily<CoverNodeProvisioning, CoverNodeId, CoverNodeMessaging>;
pub type CoverNodeIdPublicKeyFamilyList =
    IdentityPublicKeyFamilyList<CoverNodeProvisioning, CoverNodeId, CoverNodeMessaging>;

// Journalist

// Published
pub type PublishedJournalistIdPublicKeyFamily =
    UntrustedIdentityPublicKeyFamily<JournalistProvisioning, JournalistId, JournalistMessaging>;
pub type PublishedJournalistIdPublicKeyFamilyList =
    UntrustedIdentityPublicKeyFamilyList<JournalistProvisioning, JournalistId, JournalistMessaging>;

// Verified
pub type JournalistIdPublicKeyFamilyList =
    IdentityPublicKeyFamilyList<JournalistProvisioning, JournalistId, JournalistMessaging>;
pub type JournalistIdPublicKeyFamily =
    IdentityPublicKeyFamily<JournalistProvisioning, JournalistId, JournalistMessaging>;

// Backup

// Published
pub type PublishedBackupIdPublicKeyFamilyList =
    UntrustedIdentityPublicKeyFamilyList<Organization, BackupId, BackupMsg>;
pub type PublishedBackupIdPublicKeyFamily =
    UntrustedIdentityPublicKeyFamily<Organization, BackupId, BackupMsg>;

// Verified
pub type BackupIdPublicKeyFamilyList =
    IdentityPublicKeyFamilyList<Organization, BackupId, BackupMsg>;
pub type BackupIdPublicKeyFamily = IdentityPublicKeyFamily<Organization, BackupId, BackupMsg>;
