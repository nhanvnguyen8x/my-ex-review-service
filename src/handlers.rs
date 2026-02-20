//! HTTP handlers. Thin layer: extract input, call service, map to response.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::models::{CreateReview, DashboardStats, ReviewResponse};
use crate::service::ReviewService;

/// Health check endpoint.
#[utoipa::path(
    get,
    path = "/health",
    tag = "Health",
    responses((status = 200, description = "Service is healthy"))
)]
pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "my-ex-review-service"
    }))
}

/// List all reviews.
#[utoipa::path(
    get,
    path = "/reviews",
    tag = "Reviews",
    responses(
        (status = 200, description = "List of reviews", body = [ReviewResponse]),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_reviews(State(service): State<ReviewService>) -> Result<Json<Vec<ReviewResponse>>, (StatusCode, String)> {
    let list = service.list_reviews().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(list))
}

/// Get a review by ID.
#[utoipa::path(
    get,
    path = "/reviews/{id}",
    tag = "Reviews",
    params(("id" = Uuid, Path, description = "Review UUID")),
    responses(
        (status = 200, description = "Review found", body = ReviewResponse),
        (status = 404, description = "Review not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_review(State(service): State<ReviewService>, Path(id): Path<Uuid>) -> Result<Json<ReviewResponse>, (StatusCode, String)> {
    let r = service
        .get_review(id)
        .await
        .map_err(|e| if e == "Not found" { (StatusCode::NOT_FOUND, e) } else { (StatusCode::INTERNAL_SERVER_ERROR, e) })?;
    Ok(Json(r))
}

/// Create a new review.
#[utoipa::path(
    post,
    path = "/reviews",
    tag = "Reviews",
    request_body = CreateReview,
    responses(
        (status = 201, description = "Review created", body = ReviewResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_review(State(service): State<ReviewService>, Json(body): Json<CreateReview>) -> Result<(StatusCode, Json<ReviewResponse>), (StatusCode, String)> {
    let r = service.create_review(body).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok((StatusCode::CREATED, Json(r)))
}

/// Get dashboard statistics (total reviews, average rating).
#[utoipa::path(
    get,
    path = "/stats/dashboard",
    tag = "Stats",
    responses(
        (status = 200, description = "Dashboard stats", body = DashboardStats),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn dashboard_stats(State(service): State<ReviewService>) -> Result<Json<DashboardStats>, (StatusCode, String)> {
    let stats = service
        .get_dashboard_stats()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(stats))
}
