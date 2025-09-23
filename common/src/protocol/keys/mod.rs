mod generation;
mod hierarchy;
mod key_pair_with_epoch;
mod latest;
mod loading;
mod sign;
mod verification;

use crate::crypto::keys::{
    encryption::{
        PublicEncryptionKey, SignedEncryptionKeyPair, SignedPublicEncryptionKey,
        UnsignedEncryptionKeyPair,
    },
    signing::{
        PublicSigningKey, SignedPublicSigningKey, SignedSigningKeyPair, UnsignedSigningKeyPair,
    },
    untrusted::{
        encryption::{
            UntrustedPublicEncryptionKey, UntrustedSignedEncryptionKeyPair,
            UntrustedSignedPublicEncryptionKey, UntrustedUnsignedEncryptionKeyPair,
        },
        signing::{
            UntrustedPublicSigningKey, UntrustedSignedPublicSigningKey,
            UntrustedSignedSigningKeyPair, UntrustedUnsignedSigningKeyPair,
        },
    },
};

use super::roles::{
    AnchorOrganization, CoverNodeId, CoverNodeMessaging, CoverNodeProvisioning, JournalistId,
    JournalistMessaging, JournalistProvisioning, Mailbox, Organization, UnregisteredCoverNodeId,
    UnregisteredJournalistId, User,
};
pub use generation::*;
pub use hierarchy::*;
pub use key_pair_with_epoch::*;
pub use latest::*;
pub use loading::*;
pub use sign::*;
pub use verification::*;

// Type aliases for various keys used in the protocol

//
// Untrusted keys, cannot be used for cryptographic operations
//

/// A anchor organization public key that has been loaded, but has yet to have it's
/// self-signed certificate checked.
pub type UntrustedAnchorOrganizationPublicKey = UntrustedSignedPublicSigningKey<AnchorOrganization>;
pub type UntrustedOrganizationPublicKey = UntrustedSignedPublicSigningKey<Organization>;
pub type UntrustedOrganizationKeyPair = UntrustedSignedSigningKeyPair<Organization>;

impl UntrustedOrganizationPublicKey {
    /// Trust an unverified organization key.
    /// This is used when you're converting a key you've loaded from the API
    /// to be trusted (e.g. in a mailbox) in a trust-on-first-use (TOFU) manner.
    pub fn to_tofu_anchor(&self) -> UntrustedAnchorOrganizationPublicKey {
        UntrustedAnchorOrganizationPublicKey::new(
            self.key,
            self.certificate.clone(),
            self.not_valid_after,
        )
    }
}

pub type UntrustedCoverNodeProvisioningPublicKey =
    UntrustedSignedPublicSigningKey<CoverNodeProvisioning>;
pub type UntrustedCoverNodeProvisioningKeyPair =
    UntrustedSignedSigningKeyPair<CoverNodeProvisioning>;

pub type UntrustedCoverNodeIdPublicKey = UntrustedSignedPublicSigningKey<CoverNodeId>;
pub type UntrustedCoverNodeIdKeyPair = UntrustedSignedSigningKeyPair<CoverNodeId>;

pub type UntrustedUnregisteredCoverNodeIdPublicKey =
    UntrustedPublicSigningKey<UnregisteredCoverNodeId>;
pub type UntrustedUnregisteredCoverNodeIdKeyPair =
    UntrustedUnsignedSigningKeyPair<UnregisteredCoverNodeId>;

pub type UntrustedCoverNodeMessagingPublicKey =
    UntrustedSignedPublicEncryptionKey<CoverNodeMessaging>;
pub type UntrustedCoverNodeMessagingKeyPair = UntrustedSignedEncryptionKeyPair<CoverNodeMessaging>;

pub type UntrustedJournalistProvisioningPublicKey =
    UntrustedSignedPublicSigningKey<JournalistProvisioning>;
pub type UntrustedJournalistProvisioningKeyPair =
    UntrustedSignedSigningKeyPair<JournalistProvisioning>;

pub type UntrustedUnregisteredJournalistIdPublicKey =
    UntrustedPublicSigningKey<UnregisteredJournalistId>;
pub type UntrustedUnregisteredJournalistIdKeyPair =
    UntrustedUnsignedSigningKeyPair<UnregisteredJournalistId>;

pub type UntrustedJournalistIdPublicKey = UntrustedSignedPublicSigningKey<JournalistId>;
pub type UntrustedJournalistIdKeyPair = UntrustedSignedSigningKeyPair<JournalistId>;

pub type UntrustedJournalistMessagingPublicKey =
    UntrustedSignedPublicEncryptionKey<JournalistMessaging>;
pub type UntrustedJournalistMessagingKeyPair =
    UntrustedSignedEncryptionKeyPair<JournalistMessaging>;

// The user public keys are unsigned so can never truly be verified
// but having an untrusted version makes them follow the same patterns
// as all of our other keys where {de,}serialized keys are considered
// untrusted. Without this we would need to special case users keys
// within the type system.

pub type UntrustedUserPublicKey = UntrustedPublicEncryptionKey<User>;
pub type UntrustedUserKeyPair = UntrustedUnsignedEncryptionKeyPair<User>;

pub type UntrustedMailboxPublicKey = UntrustedSignedPublicEncryptionKey<Mailbox>;

//
// Verified keys
//

// A trusted organization key is used exclusively to verify a key hierarchy
// once a key hierarchy has been verified then we can safely use those keys
// to perform cryptographic operations.
//
// A trusted organization key is stored along side with application in a highly
// trusted medium, for example a secret store or on the disk with the binary.
pub type AnchorOrganizationPublicKey = SignedPublicSigningKey<AnchorOrganization>;

impl AnchorOrganizationPublicKey {
    /// Remove the trusted status of an organization key. This does not mean the key is
    /// now untrusted - it just means it can't be used to verify other public keys.
    ///
    /// This is useful when you want to use a local/trusted organization key for cryptographic
    /// purposes.
    pub fn into_non_anchor(self) -> OrganizationPublicKey {
        OrganizationPublicKey::new(self.key, self.certificate, self.not_valid_after)
    }

    /// Make a new organization key that is not trusted but is verified, this is basically a cop-out
    /// to make the type system happy but isn't used very often.
    pub fn to_non_anchor(&self) -> OrganizationPublicKey {
        OrganizationPublicKey::new(self.key, self.certificate.clone(), self.not_valid_after)
    }
}

pub type OrganizationPublicKey = SignedPublicSigningKey<Organization>;
pub type OrganizationKeyPair = SignedSigningKeyPair<Organization>;

impl OrganizationPublicKey {
    /// Upgrade a verified key into an anchor key.
    pub fn into_anchor(self) -> AnchorOrganizationPublicKey {
        AnchorOrganizationPublicKey::new(self.key, self.clone().certificate, self.not_valid_after)
    }
}

pub type CoverNodeProvisioningPublicKey = SignedPublicSigningKey<CoverNodeProvisioning>;
pub type CoverNodeProvisioningKeyPair = SignedSigningKeyPair<CoverNodeProvisioning>;
pub type CoverNodeIdPublicKey = SignedPublicSigningKey<CoverNodeId>;
pub type CoverNodeIdKeyPair = SignedSigningKeyPair<CoverNodeId>;
pub type CoverNodeMessagingPublicKey = SignedPublicEncryptionKey<CoverNodeMessaging>;
pub type CoverNodeMessagingKeyPair = SignedEncryptionKeyPair<CoverNodeMessaging>;

// CoverNode identity that has not yet been signed by a provisioning key
pub type UnregisteredCoverNodeIdPublicKey = PublicSigningKey<UnregisteredCoverNodeId>;
pub type UnregisteredCoverNodeIdKeyPair = UnsignedSigningKeyPair<UnregisteredCoverNodeId>;

pub type JournalistProvisioningPublicKey = SignedPublicSigningKey<JournalistProvisioning>;
pub type JournalistProvisioningKeyPair = SignedSigningKeyPair<JournalistProvisioning>;
pub type JournalistIdPublicKey = SignedPublicSigningKey<JournalistId>;
pub type JournalistIdKeyPair = SignedSigningKeyPair<JournalistId>;
pub type JournalistMessagingPublicKey = SignedPublicEncryptionKey<JournalistMessaging>;
pub type JournalistMessagingKeyPair = SignedEncryptionKeyPair<JournalistMessaging>;

// A journalist ID pk that has not yet been signed by a provisioning key
pub type UnregisteredJournalistIdPublicKey = PublicSigningKey<UnregisteredJournalistId>;
pub type UnregisteredJournalistIdKeyPair = UnsignedSigningKeyPair<UnregisteredJournalistId>;

pub type UserPublicKey = PublicEncryptionKey<User>;
pub type UserKeyPair = UnsignedEncryptionKeyPair<User>;

// Mailbox has only a public key because it is only used to represent a reply key from either a
// user or a journalist and has also been verified before being put in the mailbox.
pub type MailboxPublicKey = PublicEncryptionKey<Mailbox>;
