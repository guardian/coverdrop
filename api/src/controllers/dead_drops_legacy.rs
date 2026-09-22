// Legacy dead drop handlers for the deprecated /user/ and /journalist/ paths.
// TODO remove https://github.com/guardian/coverdrop-internal/issues/4202
#![allow(deprecated)]

use std::num::NonZeroU32;

use crate::cache_control::{add_cache_control_header, DEAD_DROP_TTL};
use crate::controllers::dead_drops::limit_or_default;
use crate::dead_drop_limits::DeadDropLimits;
use crate::error::AppError;
use crate::services::database::Database;
use axum::extract::{Query, State};
use axum::Json;
use common::api::models::dead_drops::{
    DeadDropId, UnverifiedJournalistToUserDeadDropsList, UnverifiedUserToJournalistDeadDropsList,
};
use http::HeaderMap;
use serde::Deserialize;

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyGetDeadDropQueryParams {
    ids_greater_than: DeadDropId,
    limit: Option<NonZeroU32>,
}

impl LegacyGetDeadDropQueryParams {
    pub fn limit_or_default(&self, default_limit: NonZeroU32) -> NonZeroU32 {
        limit_or_default(self.limit, default_limit)
    }
}

#[deprecated(note = "Use get_journalist_to_user_dead_drops instead")]
pub async fn get_user_dead_drops(
    State(db): State<Database>,
    State(dead_drop_limits): State<DeadDropLimits>,
    query_params: Query<LegacyGetDeadDropQueryParams>,
) -> Result<(HeaderMap, Json<UnverifiedJournalistToUserDeadDropsList>), AppError> {
    let ids_greater_than = query_params.ids_greater_than;
    let limit = query_params.limit_or_default(dead_drop_limits.j2u_dead_drops_per_request_limit);

    tracing::info!(
        ids_greater_than,
        "GET request for J2U dead drop with ID greater than {} limit {}",
        ids_greater_than,
        limit
    );

    let dead_drops = db
        .dead_drop_queries
        .get_journalist_to_user_dead_drops_legacy(ids_greater_than, limit)
        .await?;

    let mut headers = HeaderMap::new();
    add_cache_control_header(&mut headers, DEAD_DROP_TTL);

    Ok((
        headers,
        Json(UnverifiedJournalistToUserDeadDropsList::new(dead_drops)),
    ))
}

#[deprecated(note = "Use get_user_to_journalist_dead_drops instead")]
pub async fn get_journalist_dead_drops(
    State(db): State<Database>,
    State(dead_drop_limits): State<DeadDropLimits>,
    query_params: Query<LegacyGetDeadDropQueryParams>,
) -> Result<(HeaderMap, Json<UnverifiedUserToJournalistDeadDropsList>), AppError> {
    let ids_greater_than = query_params.ids_greater_than;
    let limit = query_params.limit_or_default(dead_drop_limits.u2j_dead_drops_per_request_limit);

    tracing::info!(
        ids_greater_than,
        "GET request for U2J dead drop with ID greater than {} limit {}",
        ids_greater_than,
        limit
    );

    let dead_drops = db
        .dead_drop_queries
        .get_user_to_journalist_dead_drops_legacy(ids_greater_than, limit)
        .await?;

    let mut headers = HeaderMap::new();
    add_cache_control_header(&mut headers, DEAD_DROP_TTL);

    Ok((
        headers,
        Json(UnverifiedUserToJournalistDeadDropsList::new(dead_drops)),
    ))
}
