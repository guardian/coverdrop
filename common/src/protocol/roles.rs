use crate::{crypto::keys::role::Role, define_role};

// Various roles for the CoverDrop system

//////////////////
// Organization //
//////////////////

// A anchor organization role is used when reading the root key off a disk
// it stands in contrast to the regular organization key which is provided by
// the PKI for the purposes of confirming your organization key is current and valid
define_role!(AnchorOrganization, "Anchor organization", "organization");

// The organization key is provided by the PKI so a client can verify their
// local, trusted, key
define_role!(Organization, "Untrusted organization", "organization");

///////////////
// CoverNode //
///////////////

// The CoverNode provisioning key is used to create CoverNode identity keys
define_role!(
    CoverNodeProvisioning,
    "CoverNode provisioning",
    "covernode_provisioning"
);

// The CoverNode identity role is used to sign CoverNode messaging keys
define_role!(CoverNodeId, "CoverNode identity", "covernode_id");

// Unsigned Covernode identity keys are used when rotating keys. They are submitted
// to the API which verifies and signs them
define_role!(
    UnregisteredCoverNodeId,
    "unsigned covernode identity",
    "unsigned_covernode_id"
);

// The CoverNode messaging key is used for communications between clients
// and the CoverNode.
define_role!(CoverNodeMessaging, "CoverNode messaging", "covernode_msg");

////////////////
// Journalist //
////////////////

// Journalist provisioning keys are used to issue new journalist identity keys
// without having to access the organization root key
define_role!(
    JournalistProvisioning,
    "journalist provisioning",
    "journalist_provisioning"
);

// Journalist identity keys are used for signing journalist messaging keys
// this allows the journalist to publish new keys daily
define_role!(JournalistId, "journalist identity", "journalist_id");

// Unsigned journalist identity keys are used when rotating keys. They are submitted
// to the API which verifies and signs them
define_role!(
    UnregisteredJournalistId,
    "unsigned journalist identity",
    "unsigned_journalist_id"
);

// The messaging key is the encryption key used by journalists to communicate with
// sources.
define_role!(
    JournalistMessaging,
    "journalist messaging",
    "journalist_msg"
);

///////////
// Users //
///////////

// The user roles is used by an anonymous source to communicate with a journalist.
define_role!(User, "user", "user");

// The mailbox role is used when storing public keys in the mailbox since we're not
// differentiating between a users send key and a journalists reply key.
define_role!(Mailbox, "mailbox", "mailbox");

/////////////
// BACKUPS //
/////////////

define_role!(
    BackupAdminEncryption,
    "backup admin encryption",
    "backup_admin_enc"
);
