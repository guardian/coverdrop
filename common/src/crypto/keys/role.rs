pub trait Role: Clone {
    fn display() -> &'static str;
    fn entity_name() -> &'static str;
}

#[macro_export]
macro_rules! define_role {
    ($name:ident, $display: tt, $entity_name: tt) => {
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
        }
    };
}

// A test role used for testing cryptographic primitives
// Used unit tests and in the admin crate to generate test vectors for cross platform testing
define_role!(Test, "Test key", "test_key");
