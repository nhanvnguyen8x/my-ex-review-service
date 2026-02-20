//! API route definitions. Group routes by domain for clarity as the service grows.

use axum::{routing::get, Router};

use crate::handlers;
use crate::service::ReviewService;

/// Health check routes.
fn health_routes() -> Router<ReviewService> {
    Router::new().route("/health", get(handlers::health))
}

/// Review CRUD routes.
fn review_routes() -> Router<ReviewService> {
    Router::new()
        .route("/reviews", get(handlers::list_reviews).post(handlers::create_review))
        .route("/reviews/:id", get(handlers::get_review))
}

/// Stats / dashboard routes.
fn stats_routes() -> Router<ReviewService> {
    Router::new().route("/stats/dashboard", get(handlers::dashboard_stats))
}

/// All API routes combined. Add new route groups here as the service grows.
pub fn api_routes() -> Router<ReviewService> {
    Router::new()
        .merge(health_routes())
        .merge(review_routes())
        .merge(stats_routes())
}
