use std::path::Path;

use chrono::{DateTime, Utc};
use common::api::models::journalist_id::JournalistIdentity;

pub fn generate_reseed_journalist_id_key_pair_form(
    _keys_path: impl AsRef<Path>,
    _journalist_id: JournalistIdentity,
    _output_path: impl AsRef<Path>,
    _now: DateTime<Utc>,
) -> anyhow::Result<()> {
    unimplemented!()
}
