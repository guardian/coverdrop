mod journalist_to_user_decrypt_and_threshold_service;
mod user_to_journalist_decrypt_and_threshold_service;

pub use journalist_to_user_decrypt_and_threshold_service::JournalistToUserDecryptionAndMixingService;
pub use user_to_journalist_decrypt_and_threshold_service::UserToJournalistDecryptionAndMixingService;

use paste::paste;

/// Convert a numeric rank to an appropraite label for the metrics system.
/// Messaging keys in the CoverNode are assigned a rank with the lower the number
/// the more recent the key.
///
/// Given that the CoverNode should only ever have 2 live messaging keys most decryption
/// attempts will only be with rank 1 or 2 keys. During key roll over there is a small window
/// where there could be a rank 3 key.
///
/// A rank 0 key is a candidate key. Successful decryption with this key indicates that
/// there is a key consensus issue and there is a key published in the API that the CoverNode
/// does not have an epoch for.
///
/// Ranks 4 and 5 are likely to only be used when attempting to decrypt with a known-to-be-expired
/// key.
///
/// While we're testing the system we're going to keep expired keys for longer than their expiry
/// time in order to see how often messages arrive which cannot be decrypted with valid keys, but
/// are using invalid keys. This will help us tune the size of our clients outbound queues.
///
/// See: https://github.com/guardian/coverdrop/issues/2974
pub fn rank_to_label(rank: usize) -> &'static str {
    match rank {
        0 => "candidate",
        1 => "1",
        2 => "2",
        3 => "3",
        4 => "4",
        5 => "5",
        _ => "older",
    }
}

/// A macro to generate record metric functions for different directions with the same structure.
/// This macro generates functions that record a counter metric with success and rank labels.
#[macro_export]
macro_rules! generate_decryption_metric_fns {
    ($fn_name:ident, $metric_name:expr) => {
        paste! {
            fn [<$fn_name _success>](rank: usize) {
                $fn_name(true, rank_to_label(rank));
            }

            fn [<$fn_name _failure>]() {
                $fn_name(false, "unknown_key");
            }

            fn $fn_name(success: bool, rank_str: &'static str) {
                let success_str = if success { "true" } else { "false" };

                metrics::counter!($metric_name, "success" => success_str, "rank" => rank_str).increment(1);
            }
        }
    };
}

generate_decryption_metric_fns!(record_u2c_metric, "U2CDecryption");
generate_decryption_metric_fns!(record_j2c_metric, "J2CDecryption");
