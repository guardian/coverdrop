use chrono::{DateTime, Duration, Utc};

use super::{
    encryption::{SignedEncryptionKeyPair, SignedPublicEncryptionKey},
    role::Role,
    signing::{SignedPublicSigningKey, SignedSigningKeyPair},
};

// Strictly speaking the role isn't required here but it helps the type inference
// which reduces the amount of manual typing we need to do on generic functions
pub trait SignedKey<R: Role> {
    fn not_valid_after(&self) -> DateTime<Utc>;

    /// Check if a public key is expired
    fn is_not_valid_after(&self, time: DateTime<Utc>) -> bool {
        self.not_valid_after() < time
    }

    /// Return the timestamp after which we notify admins that the key needs rotating
    fn rotation_notification_time(&self) -> DateTime<Utc> {
        // In general we want to notify when half the key's valid duration has elapsed.
        // We're not using *_ROTATE_AFTER constants here because journalist keys are rotated every day
        // and 24 hours before expiry is too short notice!
        // TODO rewrite once we're keeping track of key creation times.
        self.not_valid_after()
            - R::valid_duration()
                .map(|d| d / 2)
                .unwrap_or_else(|| Duration::seconds(0))
    }

    fn as_bytes(&self) -> &[u8];
}

impl<R> SignedKey<R> for SignedEncryptionKeyPair<R>
where
    R: Role,
{
    fn not_valid_after(&self) -> DateTime<Utc> {
        self.public_key().not_valid_after
    }

    fn as_bytes(&self) -> &[u8] {
        self.public_key().as_ref().as_bytes()
    }
}

impl<R> SignedKey<R> for SignedSigningKeyPair<R>
where
    R: Role,
{
    fn not_valid_after(&self) -> DateTime<Utc> {
        self.public_key().not_valid_after
    }

    fn as_bytes(&self) -> &[u8] {
        self.public_key().as_bytes()
    }
}

impl<R> SignedKey<R> for SignedPublicEncryptionKey<R>
where
    R: Role,
{
    fn not_valid_after(&self) -> DateTime<Utc> {
        self.not_valid_after
    }

    fn as_bytes(&self) -> &[u8] {
        self.as_ref().as_bytes()
    }
}

impl<R> SignedKey<R> for SignedPublicSigningKey<R>
where
    R: Role,
{
    fn not_valid_after(&self) -> DateTime<Utc> {
        self.not_valid_after
    }

    fn as_bytes(&self) -> &[u8] {
        self.as_ref().as_bytes()
    }
}

#[test]
fn test_rotation_notification_time() {
    use crate::protocol::roles::Organization;

    use crate::{
        crypto::keys::signing::UnsignedSigningKeyPair,
        protocol::constants::ORGANIZATION_KEY_VALID_DURATION,
    };

    use crate::protocol::keys::generate_covernode_provisioning_key_pair;

    let now: DateTime<Utc> = "2025-01-01T00:00:00Z".parse().unwrap();
    let org_key_expires = now + ORGANIZATION_KEY_VALID_DURATION;
    let org_key =
        UnsignedSigningKeyPair::<Organization>::generate().to_self_signed_key_pair(org_key_expires);

    let expected_notification_time: DateTime<Utc> = "2025-07-02T00:00:00Z".parse().unwrap();
    assert_eq!(
        org_key.rotation_notification_time(),
        expected_notification_time
    );

    // A provisioning key is created one month before the org key expires.
    let now: DateTime<Utc> = "2025-12-01T00:00:00Z".parse().unwrap();
    let provisioning_key = generate_covernode_provisioning_key_pair(&org_key, now);
    assert_eq!(
        provisioning_key.not_valid_after(),
        org_key.not_valid_after()
    );

    // Provisioning key notification time should be 26 weeks (provisioning key valid_duration / 2)
    // before both keys expire (and before the provisioning key is created).
    let expected_notification_time: DateTime<Utc> = "2025-07-02T00:00:00Z".parse().unwrap();
    assert_eq!(
        provisioning_key.rotation_notification_time(),
        expected_notification_time
    );
}
