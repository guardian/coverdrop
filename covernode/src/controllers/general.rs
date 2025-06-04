use axum::Json;
use common::healthcheck::HealthCheck;

pub async fn get_healthcheck() -> Json<HealthCheck> {
    let result = HealthCheck::new("covernode", "ok");

    Json(result)
}
