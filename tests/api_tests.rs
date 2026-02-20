//! Handler / API layer tests. End-to-end HTTP against the full app.
//!
//! Requires DATABASE_URL (from .env or environment). Each test gets an isolated DB.
//! Copy .env.example to .env so `cargo test` works without exporting DATABASE_URL.

use ctor::ctor;
#[ctor]
fn load_env() {
    let _ = dotenvy::dotenv();
}

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use my_ex_review_service::app;
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

/// Helper: send request to app and return (status, body as JSON).
async fn request(
    app: axum::Router<()>,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let body = body.map(|v| Body::from(serde_json::to_vec(&v).unwrap()));
    let body = body.unwrap_or_else(Body::empty);
    let req = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(body)
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = if body.is_empty() {
        json!(null)
    } else {
        serde_json::from_slice(&body).unwrap_or(json!({ "raw": String::from_utf8_lossy(&body) }))
    };
    (status, json)
}

#[sqlx::test]
async fn health_returns_ok(pool: PgPool) {
    let app = app(pool);
    let (status, body) = request(app, "GET", "/health", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
    assert_eq!(body["service"], "my-ex-review-service");
}

#[sqlx::test]
async fn list_reviews_empty(pool: PgPool) {
    let app = app(pool);
    let (status, body) = request(app, "GET", "/reviews", None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.is_array());
    assert_eq!(body.as_array().unwrap().len(), 0);
}

#[sqlx::test]
async fn get_review_not_found(pool: PgPool) {
    let app = app(pool);
    let id = Uuid::new_v4();
    let (status, _) = request(app, "GET", &format!("/reviews/{}", id), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn create_review_and_fetch(pool: PgPool) {
    let app = app(pool);
    let product_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let body = json!({
        "product_id": product_id.to_string(),
        "user_id": user_id.to_string(),
        "rating": 4,
        "body": "Great product!"
    });
    let (status, created) = request(app.clone(), "POST", "/reviews", Some(body)).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["product_id"], product_id.to_string());
    assert_eq!(created["user_id"], user_id.to_string());
    assert_eq!(created["rating"], 4);
    assert_eq!(created["body"], "Great product!");
    let id = created["id"].as_str().unwrap();

    let (status, got) = request(app, "GET", &format!("/reviews/{}", id), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(got["id"], id);
    assert_eq!(got["rating"], 4);
}

#[sqlx::test]
async fn list_reviews_after_create(pool: PgPool) {
    let app = app(pool);
    let product_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let body = json!({
        "product_id": product_id.to_string(),
        "user_id": user_id.to_string(),
        "rating": 5,
        "body": null
    });
    let (status, _) = request(app.clone(), "POST", "/reviews", Some(body)).await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, list) = request(app, "GET", "/reviews", None).await;
    assert_eq!(status, StatusCode::OK);
    let arr = list.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["rating"], 5);
}

#[sqlx::test]
async fn dashboard_stats_empty(pool: PgPool) {
    let app = app(pool);
    let (status, body) = request(app, "GET", "/stats/dashboard", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total_reviews"], 0);
    assert_eq!(body["avg_rating"], 0.0);
}

#[sqlx::test]
async fn dashboard_stats_after_reviews(pool: PgPool) {
    let app = app(pool);
    for (rating, _) in [(3, "a"), (5, "b"), (4, "c")] {
        let body = json!({
            "product_id": Uuid::new_v4().to_string(),
            "user_id": Uuid::new_v4().to_string(),
            "rating": rating,
            "body": null
        });
        let (status, _) = request(app.clone(), "POST", "/reviews", Some(body)).await;
        assert_eq!(status, StatusCode::CREATED);
    }
    let (status, body) = request(app, "GET", "/stats/dashboard", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total_reviews"], 3);
    let avg = body["avg_rating"].as_f64().unwrap();
    assert!((avg - 4.0).abs() < 0.01);
}
