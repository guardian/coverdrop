use axum::Json;
use common::healthcheck::HealthCheck;

pub async fn get_healthcheck() -> Json<HealthCheck> {
    let result = HealthCheck::new("identity-api", "ok");

    Json(result)
}
