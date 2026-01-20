// IMPORTANT: After changes to this file run `cargo run --bin admin generate-mobile-constants-files`
// to keep the mobile constants files in sync.

use chrono::Duration;

/// The time an organization key is valid for.
/// The expiry time is quite long because provisioning a new key requires
/// access to a physical machine where the secret organization key is stored
pub const ORGANIZATION_KEY_VALID_DURATION: Duration = Duration::weeks(52);

/// The amount of time between key rotations for the organization key
pub const ORGANIZATION_KEY_ROTATE_AFTER: Duration = Duration::weeks(26);

/// The amount of time the journalist provisioning is valid for
pub const JOURNALIST_PROVISIONING_KEY_VALID_DURATION: Duration = Duration::weeks(52);

/// The amount of time between key rotations for the journalist provisioning key
pub const JOURNALIST_PROVISIONING_KEY_ROTATE_AFTER: Duration = Duration::weeks(26);

/// Valid for two months in case a journalist goes on sabbatical for a month
pub const JOURNALIST_ID_KEY_VALID_DURATION: Duration = Duration::weeks(8);

/// The amount of time between key rotations for the journalist identity key
pub const JOURNALIST_ID_KEY_ROTATE_AFTER: Duration = Duration::weeks(4);

/// The time that a journalist key is valid.
///
/// In the key material this is represented as a `not_valid_after` created with the pseudocode
/// `Date.now() + JOURNALIST_MSG_KEY_VALID_DURATION`
pub const JOURNALIST_MSG_KEY_VALID_DURATION: Duration = Duration::weeks(2);

/// The amount of time between journalist messaging key rotations
pub const JOURNALIST_MSG_KEY_ROTATE_AFTER: Duration = Duration::days(1);

/// The amount of time the covernode provisioning is valid for
pub const COVERNODE_PROVISIONING_KEY_VALID_DURATION: Duration = Duration::weeks(52);

/// The amount of time between key rotations for the CoverNode provisioning key
pub const COVERNODE_PROVISIONING_KEY_ROTATE_AFTER: Duration = Duration::weeks(26);

/// CoverNode id key validity duration
pub const COVERNODE_ID_KEY_VALID_DURATION: Duration = Duration::weeks(4);

/// The amount of time between key rotations for the CoverNode identity key
pub const COVERNODE_ID_KEY_ROTATE_AFTER: Duration = Duration::weeks(2);

/// The time that the CoverNode messaging key is valid.
///
/// In the key material this is represented as a `not_valid_after` created with the pseudocode
/// `Date.now() + COVERNODE_MSG_KEY_VALID_DURATION`
pub const COVERNODE_MSG_KEY_VALID_DURATION: Duration = Duration::weeks(2);

/// The max amount of time remaining on the latest CoverNode messaging key before a new one can be uploaded.
pub const COVERNODE_MSG_KEY_ROTATE_AFTER: Duration = Duration::weeks(1);

/// The maximum time-to-live of entries within the clients' cache of the published user-facing dead-drops. The client
/// should use the most-recent dead-drop entry's timestamp as the reference of `now` to ensure that wrong local clock
/// does not lead to early evictions.
pub const CLIENT_DEAD_DROP_CACHE_TTL: Duration = Duration::weeks(2);

/// The maximum rate at which the client downloads new dead-drop entries and key hierarchies from the
/// CoverNode API.
pub const CLIENT_DEFAULT_DOWNLOAD_RATE: Duration = Duration::hours(1);

/// The maximum rate at which the client downloads status updated from the CoverNode API.
pub const CLIENT_STATUS_DOWNLOAD_RATE: Duration = Duration::minutes(5);

/// The size a message is padded to after compression.
pub const MESSAGE_PADDING_LEN: u16 = 512;

/// The number of messaging keys from different CoverNodes used when encrypting a *_TO_COVERNODE message.
pub const COVERNODE_WRAPPING_KEY_COUNT: usize = 2;

//
// USER_TO_...
//

/// The length of the message data which is sent to the CoverNode API
/// All messages must be exactly this length to make cover messages and
/// real messages indistinguishable.
///
/// This can be calculated by the following:
/// ```
/// use common::protocol::constants::*;
/// assert_eq!(USER_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN, 773);
/// assert_eq!(USER_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN,
///    COVERNODE_WRAPPING_KEY_COUNT
///    * (X25519_PUBLIC_KEY_LEN + POLY1305_AUTH_TAG_LEN + MULTI_ANONYMOUS_BOX_SECRET_KEY_LEN)
///    + USER_TO_COVERNODE_MESSAGE_LEN
///    + POLY1305_AUTH_TAG_LEN);
/// ```
///
/// The follow test will verify our constant is correct:
///
/// ```
/// use common::time;
/// use common::FixedSizeMessageText;
/// use common::api::models::journalist_id::JournalistIdentity;
/// use common::protocol::keys::test::generate_protocol_keys;
/// use common::protocol::keys::test::ProtocolKeys;
/// use common::protocol::user::encrypt_real_message_from_user_to_journalist_via_covernode;
/// use common::protocol::constants::USER_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN;
///
/// let ProtocolKeys { user_pk, hierarchy, .. } = generate_protocol_keys(time::now());
///
/// let journalist_id = JournalistIdentity::new("journalist_0").unwrap();
/// let message = FixedSizeMessageText::new("test message").unwrap();
/// let message = encrypt_real_message_from_user_to_journalist_via_covernode(
///     // The CoverDrop key hierarchy
///     &hierarchy,
///     &user_pk,
///     &journalist_id,
///     message,
/// ).unwrap();
///
/// assert_eq!(USER_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN, message.len());
/// ```
pub const USER_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN: usize = COVERNODE_WRAPPING_KEY_COUNT
    * (X25519_PUBLIC_KEY_LEN + POLY1305_AUTH_TAG_LEN + MULTI_ANONYMOUS_BOX_SECRET_KEY_LEN)
    + USER_TO_COVERNODE_MESSAGE_LEN
    + POLY1305_AUTH_TAG_LEN;

/// The length of the unencrypted outer message. Contains a real encrypted
/// message for the journalist/user or a fake cover message, as well as a tag
/// indicating either that it is a cover message or the intended recipient.
/// ```
/// use common::protocol::constants::*;
/// assert_eq!(USER_TO_COVERNODE_MESSAGE_LEN, 597);
/// assert_eq!(USER_TO_COVERNODE_MESSAGE_LEN,
///     RECIPIENT_TAG_LEN + USER_TO_JOURNALIST_ENCRYPTED_MESSAGE_LEN);
/// ```
pub const USER_TO_COVERNODE_MESSAGE_LEN: usize =
    RECIPIENT_TAG_LEN + USER_TO_JOURNALIST_ENCRYPTED_MESSAGE_LEN;

/// The length of the encrypted message for the journalist or user.
/// ```
/// use common::protocol::constants::*;
/// assert_eq!(USER_TO_JOURNALIST_ENCRYPTED_MESSAGE_LEN, 593);
/// assert_eq!(USER_TO_JOURNALIST_ENCRYPTED_MESSAGE_LEN,
///     X25519_PUBLIC_KEY_LEN + POLY1305_AUTH_TAG_LEN + (USER_TO_JOURNALIST_MESSAGE_LEN));
/// ```
pub const USER_TO_JOURNALIST_ENCRYPTED_MESSAGE_LEN: usize =
    X25519_PUBLIC_KEY_LEN + POLY1305_AUTH_TAG_LEN + USER_TO_JOURNALIST_MESSAGE_LEN;

/// The length of the padded inner message for the journalist or user. This includes space for a
/// reply public key by default.
/// ```
/// use common::protocol::constants::*;
/// assert_eq!(USER_TO_JOURNALIST_MESSAGE_LEN, 545);
/// assert_eq!(USER_TO_JOURNALIST_MESSAGE_LEN,
///     X25519_PUBLIC_KEY_LEN + USER_TO_JOURNALIST_MESSAGE_RESERVED_BYTE + MESSAGE_PADDING_LEN as usize);
/// ```
pub const USER_TO_JOURNALIST_MESSAGE_LEN: usize =
    X25519_PUBLIC_KEY_LEN + USER_TO_JOURNALIST_MESSAGE_RESERVED_BYTE + MESSAGE_PADDING_LEN as usize;
pub const USER_TO_JOURNALIST_MESSAGE_RESERVED_BYTE: usize = 1;

//
// COVERNODE_TO_...
//

/// Length of the encrypted message sent from the CoverNode to the journalists' dead drop. This
/// includes the overhead for the TwoPartyBox and the length of the wrapped
/// CoverNodeToJournalistMessage.
///
/// ```
/// use common::protocol::constants::*;
/// assert_eq!(COVERNODE_TO_JOURNALIST_ENCRYPTED_MESSAGE_LEN, 633);
/// assert_eq!(COVERNODE_TO_JOURNALIST_ENCRYPTED_MESSAGE_LEN,
///     TWO_PARTY_BOX_NONCE_LEN + POLY1305_AUTH_TAG_LEN + USER_TO_JOURNALIST_ENCRYPTED_MESSAGE_LEN);
/// ```
pub const COVERNODE_TO_JOURNALIST_ENCRYPTED_MESSAGE_LEN: usize =
    TWO_PARTY_BOX_NONCE_LEN + POLY1305_AUTH_TAG_LEN + COVERNODE_TO_JOURNALIST_MESSAGE_LEN;

/// The length unencrypted of the message CoverNode to the journalists' dead drop just includes
/// the UserToJournalistMessage as payload.
///
/// ```
/// use common::protocol::constants::*;
/// assert_eq!(COVERNODE_TO_JOURNALIST_MESSAGE_LEN, 593);
/// assert_eq!(COVERNODE_TO_JOURNALIST_MESSAGE_LEN,USER_TO_JOURNALIST_ENCRYPTED_MESSAGE_LEN);
/// ```
pub const COVERNODE_TO_JOURNALIST_MESSAGE_LEN: usize = USER_TO_JOURNALIST_ENCRYPTED_MESSAGE_LEN;

//
// JOURNALIST_TO_...
//

/// The length of the message data which is sent to the CoverNode API
/// All messages must be exactly this length to make cover messages and
/// real messages indistinguishable.
///
/// This can be calculated by the following:
/// ```
/// use common::protocol::constants::*;
/// assert_eq!(JOURNALIST_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN, 730);
/// assert_eq!(JOURNALIST_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN,
///    COVERNODE_WRAPPING_KEY_COUNT
///    * (X25519_PUBLIC_KEY_LEN + POLY1305_AUTH_TAG_LEN + MULTI_ANONYMOUS_BOX_SECRET_KEY_LEN)
///    + JOURNALIST_TO_COVERNODE_MESSAGE_LEN
///    + POLY1305_AUTH_TAG_LEN);
/// ```
///
/// The follow test will verify our constant is correct:
///
/// ```
/// use common::time;
/// use common::FixedSizeMessageText;
/// use common::protocol::keys::test::generate_protocol_keys;
/// use common::protocol::keys::test::ProtocolKeys;
/// use common::protocol::journalist::encrypt_real_message_from_journalist_to_user_via_covernode;
/// use common::protocol::constants::JOURNALIST_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN;
///
/// let ProtocolKeys { user_pk, journalist_msg_key_pair, hierarchy, .. } = generate_protocol_keys(time::now());
///
/// let message = FixedSizeMessageText::new("test message").unwrap();
/// let message = encrypt_real_message_from_journalist_to_user_via_covernode(
///     // The message key for the CoverNode, used in the outer layer of encryption
///     &hierarchy,
///     // The user's public key, used for the inner layer of encryption
///     &user_pk,
///     // The journalist's key pair, used for the inner layer of encryption
///     &journalist_msg_key_pair,
///     &message,
/// ).unwrap();
///
/// assert_eq!(JOURNALIST_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN, message.len());
/// ```
pub const JOURNALIST_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN: usize = COVERNODE_WRAPPING_KEY_COUNT
    * (X25519_PUBLIC_KEY_LEN + POLY1305_AUTH_TAG_LEN + MULTI_ANONYMOUS_BOX_SECRET_KEY_LEN)
    + JOURNALIST_TO_COVERNODE_MESSAGE_LEN
    + POLY1305_AUTH_TAG_LEN;

/// The length of the unencrypted outer message. Contains a real encrypted
/// message for the journalist/user or a fake cover message, as well as a byte
/// flagging whether the message is real or fake.
/// ```
/// use common::protocol::constants::*;
/// assert_eq!(JOURNALIST_TO_COVERNODE_MESSAGE_LEN, 554);
/// assert_eq!(JOURNALIST_TO_COVERNODE_MESSAGE_LEN,
///     REAL_OR_COVER_BYTE_LEN + JOURNALIST_TO_USER_ENCRYPTED_MESSAGE_LEN);
/// ```
pub const JOURNALIST_TO_COVERNODE_MESSAGE_LEN: usize =
    REAL_OR_COVER_BYTE_LEN + JOURNALIST_TO_USER_ENCRYPTED_MESSAGE_LEN;

/// The length of the encrypted message for the journalist or user.
/// ```
/// use common::protocol::constants::*;
/// assert_eq!(JOURNALIST_TO_USER_ENCRYPTED_MESSAGE_LEN, 553);
/// assert_eq!(JOURNALIST_TO_USER_ENCRYPTED_MESSAGE_LEN,
///     POLY1305_AUTH_TAG_LEN + JOURNALIST_TO_USER_MESSAGE_LEN + TWO_PARTY_BOX_NONCE_LEN);
/// ```
pub const JOURNALIST_TO_USER_ENCRYPTED_MESSAGE_LEN: usize =
    POLY1305_AUTH_TAG_LEN + JOURNALIST_TO_USER_MESSAGE_LEN + TWO_PARTY_BOX_NONCE_LEN;

/// The length of the padded inner message for the journalist or user. This includes space for a
/// reply public key by default.
/// ```
/// use common::protocol::constants::*;
/// assert_eq!(JOURNALIST_TO_USER_MESSAGE_LEN, 513);
/// assert_eq!(JOURNALIST_TO_USER_MESSAGE_LEN,
///     JOURNALIST_TO_USER_MESSAGE_TYPE_FLAG_LEN +
///     MESSAGE_PADDING_LEN as usize);
/// ```
pub const JOURNALIST_TO_USER_MESSAGE_LEN: usize =
    JOURNALIST_TO_USER_MESSAGE_TYPE_FLAG_LEN + MESSAGE_PADDING_LEN as usize;
pub const JOURNALIST_TO_USER_MESSAGE_TYPE_FLAG_LEN: usize = 1;

//
// Constants just to make the above more readable, will actually be verified by the doc test.
//

pub const REAL_OR_COVER_BYTE_LEN: usize = 1;

// SAFETY: This tag must shorter SHA256 hash length (32 bytes)
// This is because the tag is created by truncating a SHA256 hash of the journalist ID
// so that it is (1) a fixed size and (2) short
pub const RECIPIENT_TAG_LEN: usize = 4;
const _: () = assert!(RECIPIENT_TAG_LEN < 32);

pub const ED25519_PUBLIC_KEY_LEN: usize = 32;
pub const ED25519_SECRET_KEY_LEN: usize = 32;
pub const X25519_PUBLIC_KEY_LEN: usize = 32;
pub const X25519_SECRET_KEY_LEN: usize = 32;
pub const POLY1305_AUTH_TAG_LEN: usize = 16;
pub const TWO_PARTY_BOX_NONCE_LEN: usize = 24;
pub const MULTI_ANONYMOUS_BOX_SECRET_KEY_LEN: usize = 32;

//
// App related durations
//

/// Messages are valid for 14 days from their sent / received time
pub const MESSAGE_VALID_FOR_DURATION: Duration = Duration::days(14);

/// Users are warned their messages will expire up to 48 hours before they expire
pub const MESSAGE_EXPIRY_WARNING: Duration = Duration::days(2);

/// Maximum time the user can background the app before they are logged out
pub const MAX_BACKGROUND_DURATION: Duration = Duration::minutes(5);

//
// Journalist vault backup constants
//
// Keep in line with journalist-client/src/constants.ts
// It's not currently possible to export constants via ts-rs, so we re-declare them here.
// See https://github.com/Aleph-Alpha/ts-rs/issues/441

// The minimum number of shares required to reconstruct the backup secret. Must be <= N.
pub const SECRET_SHARING_K_VALUE: usize = 1;

// The total number of shares to create for the backup secret. Must be >= K.
pub const SECRET_SHARING_N_VALUE: usize = 1;
