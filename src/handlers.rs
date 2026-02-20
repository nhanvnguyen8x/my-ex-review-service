use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::models::{CreateReview, DashboardStats, Review, ReviewResponse};

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
pub async fn list_reviews(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<ReviewResponse>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, Review>(
        "SELECT id, product_id, user_id, rating, body, created_at FROM reviews ORDER BY created_at DESC",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(
        rows.into_iter()
            .map(|r| ReviewResponse {
                id: r.id,
                product_id: r.product_id,
                user_id: r.user_id,
                rating: r.rating,
                body: r.body.clone(),
                created_at: r.created_at,
            })
            .collect(),
    ))
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
pub async fn get_review(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<ReviewResponse>, (StatusCode, String)> {
    let r = sqlx::query_as::<_, Review>(
        "SELECT id, product_id, user_id, rating, body, created_at FROM reviews WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Not found".to_string()))?;
    Ok(Json(ReviewResponse {
        id: r.id,
        product_id: r.product_id,
        user_id: r.user_id,
        rating: r.rating,
        body: r.body.clone(),
        created_at: r.created_at,
    }))
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
pub async fn create_review(
    State(pool): State<PgPool>,
    Json(body): Json<CreateReview>,
) -> Result<(StatusCode, Json<ReviewResponse>), (StatusCode, String)> {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO reviews (id, product_id, user_id, rating, body) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(id)
    .bind(body.product_id)
    .bind(body.user_id)
    .bind(body.rating)
    .bind(body.body)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let r = sqlx::query_as::<_, Review>(
        "SELECT id, product_id, user_id, rating, body, created_at FROM reviews WHERE id = $1",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((
        StatusCode::CREATED,
        Json(ReviewResponse {
            id: r.id,
            product_id: r.product_id,
            user_id: r.user_id,
            rating: r.rating,
            body: r.body.clone(),
            created_at: r.created_at,
        }),
    ))
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
pub async fn dashboard_stats(
    State(pool): State<PgPool>,
) -> Result<Json<DashboardStats>, (StatusCode, String)> {
    let row = sqlx::query("SELECT COUNT(*) as count FROM reviews")
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let total: i64 = row.get::<i64, _>("count");

    let row = sqlx::query("SELECT COALESCE(AVG(rating), 0)::float8 as avg FROM reviews")
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let avg_rating: f64 = row.get::<f64, _>("avg");

    Ok(Json(DashboardStats {
        total_reviews: total as u64,
        avg_rating,
    }))
}
