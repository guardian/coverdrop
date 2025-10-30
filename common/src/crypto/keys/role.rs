pub trait Role: Clone {
    fn display() -> &'static str;
    fn entity_name() -> &'static str;
    fn valid_duration_seconds() -> Option<i64>;
    fn rotate_after_seconds() -> Option<i64>;
}

#[macro_export]
macro_rules! define_role {
    ($name:ident, $display: tt, $entity_name: tt, $valid_duration_seconds: expr, $rotate_after_seconds: expr) => {
        #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, Hash)]
        #[serde(deny_unknown_fields)]
        pub struct $name {}

        impl Role for $name {
            /// A human readable name for this role
            fn display() -> &'static str {
                $display
            }

            /// A name for this role which follows conventions to use in databases or file systems.
            /// For example, it must not contain spaces or uppercase letters
            fn entity_name() -> &'static str {
                $entity_name
            }

            /// Return the duration in seconds for which keys of this role are valid
            fn valid_duration_seconds() -> Option<i64> {
                $valid_duration_seconds
            }

            /// Return the duration in seconds after which keys of this role should rotate
            fn rotate_after_seconds() -> Option<i64> {
                $rotate_after_seconds
            }
        }
    };
    ($name:ident, $display: tt, $entity_name: tt) => {
        $crate::define_role!($name, $display, $entity_name, None, None);
    };
}

// A test role used for testing cryptographic primitives without valid duration or rotation time
// Used unit tests and in the admin crate to generate test vectors for cross platform testing
define_role!(Test, "Test key", "test_key");

// A test role with a valid duration and rotation time
define_role!(
    Test2,
    "Test key 2",
    "test_key_with_rotate_after",
    Some(60 * 60 * 24),
    Some(60 * 60 * 12)
);
