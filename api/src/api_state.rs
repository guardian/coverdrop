use crate::anchor_org_pk_cache::AnchorOrganizationPublicKeyCache;
use crate::dead_drop_limits::DeadDropLimits;
use crate::services::database::Database;
use axum::extract::FromRef;
use common::api::models::journalist_id::JournalistIdentity;
use common::aws::kinesis::client::KinesisClient;
use common::aws::s3::client::S3Client;
use common::tracing::TracingReloadHandle;

#[derive(Clone, FromRef)]
pub struct ApiState {
    pub anchor_org_pks: AnchorOrganizationPublicKeyCache,
    pub db: Database,
    pub kinesis_client: KinesisClient,
    pub s3_client: S3Client,
    pub default_journalist_id: Option<JournalistIdentity>,
    pub tracing_reload_handle: TracingReloadHandle,
    pub dead_drop_limits: DeadDropLimits,
}

impl ApiState {
    pub fn new(
        anchor_org_pks: AnchorOrganizationPublicKeyCache,
        db: Database,
        kinesis_client: KinesisClient,
        s3_client: S3Client,
        default_journalist_id: Option<JournalistIdentity>,
        tracing_reload_handle: TracingReloadHandle,
        dead_drop_limits: DeadDropLimits,
    ) -> Self {
        ApiState {
            anchor_org_pks,
            db,
            kinesis_client,
            s3_client,
            default_journalist_id,
            tracing_reload_handle,
            dead_drop_limits,
        }
    }
}
