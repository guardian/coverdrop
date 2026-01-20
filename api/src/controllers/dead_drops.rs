use std::num::NonZeroU32;

use crate::anchor_org_pk_cache::AnchorOrganizationPublicKeyCache;
use crate::cache_control::{add_cache_control_header, DEAD_DROP_TTL};
use crate::dead_drop_limits::DeadDropLimits;
use crate::error::AppError;
use crate::services::database::Database;
use axum::extract::{Query, State};
use axum::Json;
use common::api::models::dead_drop_summary::DeadDropSummary;
use common::api::models::dead_drops::{
    DeadDropId, UnpublishedJournalistToUserDeadDrop, UnpublishedUserToJournalistDeadDrop,
    UnverifiedJournalistToUserDeadDropsList, UnverifiedUserToJournalistDeadDropsList,
};
use common::protocol::covernode::{
    verify_unpublished_journalist_to_user_dead_drop,
    verify_unpublished_user_to_journalist_dead_drop,
};
use common::time;
use http::HeaderMap;
use serde::Deserialize;

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GetDeadDropQueryParams {
    ids_greater_than: DeadDropId,
    limit: Option<NonZeroU32>,
}

impl GetDeadDropQueryParams {
    pub fn limit_or_default(&self, default_limit: NonZeroU32) -> NonZeroU32 {
        self.limit.map_or(default_limit, |requested_limit| {
            if requested_limit.get() <= default_limit.get() {
                requested_limit
            } else {
                tracing::warn!(
                    "Request made for {} dead drops, which is more than the dead drop limit ({})",
                    requested_limit,
                    default_limit
                );
                default_limit
            }
        })
    }
}

pub async fn get_user_dead_drops(
    State(db): State<Database>,
    State(dead_drop_limits): State<DeadDropLimits>,
    query_params: Query<GetDeadDropQueryParams>,
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
        .get_journalist_to_user_dead_drops(ids_greater_than, limit)
        .await?;

    let mut headers = HeaderMap::new();
    add_cache_control_header(&mut headers, DEAD_DROP_TTL);

    Ok((
        headers,
        Json(UnverifiedJournalistToUserDeadDropsList::new(dead_drops)),
    ))
}

pub async fn get_journalist_dead_drops(
    State(db): State<Database>,
    State(dead_drop_limits): State<DeadDropLimits>,
    query_params: Query<GetDeadDropQueryParams>,
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
        .get_user_to_journalist_dead_drops(ids_greater_than, limit)
        .await?;

    let mut headers = HeaderMap::new();
    add_cache_control_header(&mut headers, DEAD_DROP_TTL);

    Ok((
        headers,
        Json(UnverifiedUserToJournalistDeadDropsList::new(dead_drops)),
    ))
}

pub async fn post_user_dead_drops(
    State(anchor_org_pks): State<AnchorOrganizationPublicKeyCache>,
    State(db): State<Database>,
    Json(dead_drop): Json<UnpublishedJournalistToUserDeadDrop>,
) -> Result<(), AppError> {
    let (keys, _max_epoch) = db
        .hierarchy_queries
        .key_hierarchy(&anchor_org_pks.get().await, time::now())
        .await?;

    let dead_drop = verify_unpublished_journalist_to_user_dead_drop(&keys, dead_drop, time::now())
        .map_err(|_| AppError::SignatureVerificationFailed)?;

    let id = db
        .dead_drop_queries
        .add_journalist_to_user_dead_drop(dead_drop, time::now())
        .await?;

    tracing::info!("Successfully added J2U dead drop {}", id);

    Ok(())
}

pub async fn post_journalist_dead_drops(
    State(anchor_org_pks): State<AnchorOrganizationPublicKeyCache>,
    State(db): State<Database>,
    Json(dead_drop): Json<UnpublishedUserToJournalistDeadDrop>,
) -> Result<(), AppError> {
    let (keys, _max_epoch) = db
        .hierarchy_queries
        .key_hierarchy(&anchor_org_pks.get().await, time::now())
        .await?;

    let dead_drop = verify_unpublished_user_to_journalist_dead_drop(&keys, dead_drop, time::now())
        .map_err(|_| AppError::SignatureVerificationFailed)?;

    let id = db
        .dead_drop_queries
        .add_user_to_journalist_dead_drop(dead_drop, time::now())
        .await?;

    tracing::info!("Successfully added U2J dead drop {}", id);

    Ok(())
}

pub async fn get_user_recent_dead_drop_summary(
    State(db): State<Database>,
) -> Result<(HeaderMap, Json<Vec<DeadDropSummary>>), AppError> {
    let summaries = db
        .dead_drop_queries
        .get_journalist_to_user_recent_dead_drop_summary()
        .await?;

    let mut headers = HeaderMap::new();
    add_cache_control_header(&mut headers, DEAD_DROP_TTL);

    Ok((headers, Json(summaries)))
}

pub async fn get_journalist_recent_dead_drop_summary(
    State(db): State<Database>,
) -> Result<(HeaderMap, Json<Vec<DeadDropSummary>>), AppError> {
    let summaries = db
        .dead_drop_queries
        .get_user_to_journalist_recent_dead_drop_summary()
        .await?;

    let mut headers = HeaderMap::new();
    add_cache_control_header(&mut headers, DEAD_DROP_TTL);

    Ok((headers, Json(summaries)))
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    use super::GetDeadDropQueryParams;

    // Wrapper to avoid the construction of a GetDeadDropQueryParams many times
    fn query_param_limit(limit: Option<NonZeroU32>, default_limit: NonZeroU32) -> u32 {
        let params = GetDeadDropQueryParams {
            ids_greater_than: 1,
            limit,
        };
        params.limit_or_default(default_limit).get()
    }

    #[test]
    fn query_params_limit_less_than_default() {
        assert_eq!(
            query_param_limit(
                Some(NonZeroU32::new(1).unwrap()),
                NonZeroU32::new(10).unwrap()
            ),
            1
        );
    }

    #[test]
    fn query_params_limit_equal_to_default() {
        assert_eq!(
            query_param_limit(
                Some(NonZeroU32::new(10).unwrap()),
                NonZeroU32::new(10).unwrap()
            ),
            10
        );
    }

    #[test]
    fn query_params_limit_more_than_default() {
        assert_eq!(
            query_param_limit(
                Some(NonZeroU32::new(100).unwrap()),
                NonZeroU32::new(10).unwrap()
            ),
            10
        );
    }
}
