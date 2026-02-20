//! Service layer tests. Exercise business logic and mapping (service + repository).
//!
//! Requires DATABASE_URL (from .env or environment). Copy .env.example to .env for `cargo test`.

use ctor::ctor;
#[ctor]
fn load_env() {
    let _ = dotenvy::dotenv();
}

use my_ex_review_service::models::CreateReview;
use my_ex_review_service::service::ReviewService;
use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test]
async fn list_reviews_empty(pool: PgPool) {
    let service = ReviewService::new(pool);
    let list = service.list_reviews().await.unwrap();
    assert!(list.is_empty());
}

#[sqlx::test]
async fn get_review_not_found(pool: PgPool) {
    let service = ReviewService::new(pool);
    let err = service.get_review(Uuid::new_v4()).await.unwrap_err();
    assert_eq!(err, "Not found");
}

#[sqlx::test]
async fn create_review_returns_response(pool: PgPool) {
    let service = ReviewService::new(pool);
    let body = CreateReview {
        product_id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        rating: 5,
        body: Some("great".to_string()),
    };
    let r = service.create_review(body).await.unwrap();
    assert_eq!(r.rating, 5);
    assert_eq!(r.body.as_deref(), Some("great"));
}

#[sqlx::test]
async fn create_then_get_review(pool: PgPool) {
    let service = ReviewService::new(pool);
    let body = CreateReview {
        product_id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        rating: 4,
        body: None,
    };
    let created = service.create_review(body).await.unwrap();
    let got = service.get_review(created.id).await.unwrap();
    assert_eq!(got.id, created.id);
    assert_eq!(got.rating, 4);
}

#[sqlx::test]
async fn list_reviews_after_create(pool: PgPool) {
    let service = ReviewService::new(pool);
    let body = CreateReview {
        product_id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        rating: 3,
        body: None,
    };
    service.create_review(body).await.unwrap();
    let list = service.list_reviews().await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].rating, 3);
}

#[sqlx::test]
async fn get_dashboard_stats_empty(pool: PgPool) {
    let service = ReviewService::new(pool);
    let stats = service.get_dashboard_stats().await.unwrap();
    assert_eq!(stats.total_reviews, 0);
    assert!((stats.avg_rating - 0.0).abs() < 1e-9);
}

#[sqlx::test]
async fn get_dashboard_stats_after_reviews(pool: PgPool) {
    let service = ReviewService::new(pool);
    for rating in [3, 5, 4] {
        let body = CreateReview {
            product_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            rating,
            body: None,
        };
        service.create_review(body).await.unwrap();
    }
    let stats = service.get_dashboard_stats().await.unwrap();
    assert_eq!(stats.total_reviews, 3);
    assert!((stats.avg_rating - 4.0).abs() < 0.01);
}
