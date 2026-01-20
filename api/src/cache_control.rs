use axum::http::HeaderValue;
use chrono::Duration;
use http::{header::CACHE_CONTROL, HeaderMap};

pub const DEAD_DROP_TTL: Duration = Duration::minutes(5);
pub const PUBLIC_KEYS_TTL: Duration = Duration::minutes(1);
pub const STATUS_TTL: Duration = Duration::seconds(5);
pub const HEALTHCHECK_TTL: Duration = Duration::seconds(1);
pub const ROTATION_FORM_TTL: Duration = Duration::seconds(5);

/// Insert the header `cache-control: max-age=ttl` into a header map.
/// The TTL is converted into seconds
pub fn add_cache_control_header(header_map: &mut HeaderMap, ttl: Duration) {
    let header_value = format!("max-age={}", ttl.num_seconds());
    let header_value = HeaderValue::from_str(&header_value).unwrap();
    header_map.insert(CACHE_CONTROL, header_value);
}
