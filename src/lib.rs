//! Library entry point for the review service. Exposes the app router for testing and reuse.

use axum::Router;
use sqlx::PgPool;
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub mod handlers;
pub mod models;
pub mod repository;
pub mod routes;
pub mod service;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::health,
        handlers::list_reviews,
        handlers::get_review,
        handlers::create_review,
        handlers::dashboard_stats,
    ),
    components(schemas(
        crate::models::CreateReview,
        crate::models::ReviewResponse,
        crate::models::DashboardStats,
    )),
    info(
        title = "My EX Review Service API",
        version = "1.0.0",
        description = "Review / Analytics microservice API",
    ),
    tags(
        (name = "Health", description = "Health check"),
        (name = "Reviews", description = "Review CRUD"),
        (name = "Stats", description = "Dashboard statistics"),
    )
)]
struct ApiDoc;

/// Build the application router with the given database pool.
/// Used by the binary and by integration tests.
pub fn app(pool: PgPool) -> Router<()> {
    let review_service = service::ReviewService::new(pool);
    routes::api_routes()
        .merge(Router::<service::ReviewService>::from(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi())))
        .layer(CorsLayer::permissive())
        .with_state(review_service)
}
