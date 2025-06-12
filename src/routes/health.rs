#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "API Health", body = String)
    ),
    tag = "health"
)]
pub async fn health_check() -> String {
    "Okay".to_string()
}
