use axum::Json;
use common::healthcheck::HealthCheck;

pub async fn get_healthcheck() -> Json<HealthCheck> {
    let result = HealthCheck::new("delivery-service", "ok");
    Json(result)
}
