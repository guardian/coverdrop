use crate::{
    define_role,
    protocol::constants::{
        COVERNODE_ID_KEY_ROTATE_AFTER, COVERNODE_ID_KEY_VALID_DURATION,
        COVERNODE_MSG_KEY_ROTATE_AFTER, COVERNODE_MSG_KEY_VALID_DURATION,
        COVERNODE_PROVISIONING_KEY_ROTATE_AFTER, COVERNODE_PROVISIONING_KEY_VALID_DURATION,
        JOURNALIST_ID_KEY_ROTATE_AFTER, JOURNALIST_ID_KEY_VALID_DURATION,
        JOURNALIST_MSG_KEY_ROTATE_AFTER, JOURNALIST_MSG_KEY_VALID_DURATION,
        JOURNALIST_PROVISIONING_KEY_ROTATE_AFTER, JOURNALIST_PROVISIONING_KEY_VALID_DURATION,
        ORGANIZATION_KEY_ROTATE_AFTER, ORGANIZATION_KEY_VALID_DURATION,
    },
};

// Various roles for the CoverDrop system

//////////////////
// Organization //
//////////////////

// An anchor organization role is used when reading the root key off a disk
// it stands in contrast to the regular organization key which is provided by
// the PKI for the purposes of confirming your organization key is current and valid
define_role!(
    AnchorOrganization,
    "Anchor organization",
    "organization",
    Some(ORGANIZATION_KEY_VALID_DURATION),
    Some(ORGANIZATION_KEY_ROTATE_AFTER)
);

// The organization key is provided by the PKI so a client can verify their
// local, trusted, key
define_role!(
    Organization,
    "Untrusted organization",
    "organization",
    Some(ORGANIZATION_KEY_VALID_DURATION),
    Some(ORGANIZATION_KEY_ROTATE_AFTER)
);

///////////////
// CoverNode //
///////////////

// The CoverNode provisioning key is used to create CoverNode identity keys
define_role!(
    CoverNodeProvisioning,
    "CoverNode provisioning",
    "covernode_provisioning",
    Some(COVERNODE_PROVISIONING_KEY_VALID_DURATION),
    Some(COVERNODE_PROVISIONING_KEY_ROTATE_AFTER)
);

// The CoverNode identity role is used to sign CoverNode messaging keys
define_role!(
    CoverNodeId,
    "CoverNode identity",
    "covernode_id",
    Some(COVERNODE_ID_KEY_VALID_DURATION),
    Some(COVERNODE_ID_KEY_ROTATE_AFTER)
);

// Unsigned Covernode identity keys are used when rotating keys. They are submitted
// to the API which verifies and signs them
define_role!(
    UnregisteredCoverNodeId,
    "unsigned covernode identity",
    "unsigned_covernode_id",
    Some(COVERNODE_ID_KEY_VALID_DURATION),
    Some(COVERNODE_ID_KEY_ROTATE_AFTER)
);

// The CoverNode messaging key is used for communications between clients
// and the CoverNode.
define_role!(
    CoverNodeMessaging,
    "CoverNode messaging",
    "covernode_msg",
    Some(COVERNODE_MSG_KEY_VALID_DURATION),
    Some(COVERNODE_MSG_KEY_ROTATE_AFTER)
);

////////////////
// Journalist //
////////////////

// Journalist provisioning keys are used to issue new journalist identity keys
// without having to access the organization root key
define_role!(
    JournalistProvisioning,
    "journalist provisioning",
    "journalist_provisioning",
    Some(JOURNALIST_PROVISIONING_KEY_VALID_DURATION),
    Some(JOURNALIST_PROVISIONING_KEY_ROTATE_AFTER)
);

// Journalist identity keys are used for signing journalist messaging keys
// this allows the journalist to publish new keys daily
define_role!(
    JournalistId,
    "journalist identity",
    "journalist_id",
    Some(JOURNALIST_ID_KEY_VALID_DURATION),
    Some(JOURNALIST_ID_KEY_ROTATE_AFTER)
);

// Unsigned journalist identity keys are used when rotating keys. They are submitted
// to the API which verifies and signs them
define_role!(
    UnregisteredJournalistId,
    "unsigned journalist identity",
    "unsigned_journalist_id",
    Some(JOURNALIST_ID_KEY_VALID_DURATION),
    Some(JOURNALIST_ID_KEY_ROTATE_AFTER)
);

// The messaging key is the encryption key used by journalists to communicate with
// sources.
define_role!(
    JournalistMessaging,
    "journalist messaging",
    "journalist_msg",
    Some(JOURNALIST_MSG_KEY_VALID_DURATION),
    Some(JOURNALIST_MSG_KEY_ROTATE_AFTER)
);

///////////
// Users //
///////////

// The user roles is used by an anonymous source to communicate with a journalist.
define_role!(User, "user", "user");

// The mailbox role is used when storing public keys in the mailbox since we're not
// differentiating between a users send key and a journalists reply key.
define_role!(Mailbox, "mailbox", "mailbox");
