use chrono::Duration;
use std::fmt::Debug;

pub trait Role: Clone {
    type CertData: Clone + Debug + PartialEq + Eq;

    fn display() -> &'static str;
    fn entity_name() -> &'static str;
    fn valid_duration() -> Option<Duration>;
    fn rotate_after() -> Option<Duration>;
}

#[macro_export]
macro_rules! define_role {
    // Allow defining a role with a specific certificate data type.
    // This is used to differentiate signing key roles for identity keys, which need to include
    // identities in their certificate data, from roles that use the default `KeyCertificateData`.
    // Encryption key roles should use the default `KeyCertificateData`.
    ($name:ident, $display: tt, $entity_name: tt, $valid_duration: expr, $rotate_after: expr, $cert_data: ty) => {
        #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, Hash)]
        #[serde(deny_unknown_fields)]
        pub struct $name {}

        impl $crate::crypto::keys::role::Role for $name {
            type CertData = $cert_data;

            /// A human-readable name for this role
            fn display() -> &'static str {
                $display
            }

            /// A name for this role which follows conventions to use in databases or file systems.
            /// For example, it must not contain spaces or uppercase letters
            fn entity_name() -> &'static str {
                $entity_name
            }

            /// Return the duration for which keys of this role are valid
            fn valid_duration() -> Option<chrono::Duration> {
                $valid_duration
            }

            /// Return the duration after which keys of this role should rotate
            fn rotate_after() -> Option<chrono::Duration> {
                $rotate_after
            }
        }
    };
    // If no certificate data type is provided, default to using `KeyCertificateData`
    ($name:ident, $display: tt, $entity_name: tt, $valid_duration: expr, $rotate_after: expr) => {
        $crate::define_role!(
            $name,
            $display,
            $entity_name,
            $valid_duration,
            $rotate_after,
            $crate::crypto::keys::key_certificate_data::KeyCertificateData
        );
    };
    ($name:ident, $display: tt, $entity_name: tt) => {
        $crate::define_role!($name, $display, $entity_name, None, None);
    };
}

// A test role used for testing cryptographic primitives without valid duration or rotation time
// Used unit tests and in the admin crate to generate test vectors for cross-platform testing
define_role!(Test, "Test key", "test_key");
