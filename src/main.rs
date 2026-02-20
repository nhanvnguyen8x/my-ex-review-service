use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod handlers;
mod models;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/review_db".to_string()
        }))
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let app = Router::new()
        .route("/health", get(handlers::health))
        .route("/reviews", get(handlers::list_reviews).post(handlers::create_review))
        .route("/reviews/:id", get(handlers::get_review))
        .route("/stats/dashboard", get(handlers::dashboard_stats))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(CorsLayer::permissive())
        .with_state(pool);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3005);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Review service listening on {}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    Ok(())
}
