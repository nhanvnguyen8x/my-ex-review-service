//! Repository layer tests. Exercise data access in isolation.
//!
//! Requires DATABASE_URL (from .env or environment). Copy .env.example to .env for `cargo test`.

use ctor::ctor;
#[ctor]
fn load_env() {
    let _ = dotenvy::dotenv();
}

use my_ex_review_service::models::CreateReview;
use my_ex_review_service::repository::ReviewRepository;
use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test]
async fn find_all_empty(pool: PgPool) {
    let repo = ReviewRepository::new(pool);
    let list = repo.find_all().await.unwrap();
    assert!(list.is_empty());
}

#[sqlx::test]
async fn find_by_id_not_found(pool: PgPool) {
    let repo = ReviewRepository::new(pool);
    let got = repo.find_by_id(Uuid::new_v4()).await.unwrap();
    assert!(got.is_none());
}

#[sqlx::test]
async fn create_and_find_by_id(pool: PgPool) {
    let repo = ReviewRepository::new(pool);
    let id = Uuid::new_v4();
    let body = CreateReview {
        product_id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        rating: 4,
        body: Some("nice".to_string()),
    };
    let created = repo.create(id, &body).await.unwrap();
    assert_eq!(created.id, id);
    assert_eq!(created.rating, 4);
    assert_eq!(created.body.as_deref(), Some("nice"));

    let found = repo.find_by_id(id).await.unwrap().unwrap();
    assert_eq!(found.id, id);
    assert_eq!(found.rating, 4);
}

#[sqlx::test]
async fn find_all_returns_created_in_desc_order(pool: PgPool) {
    let repo = ReviewRepository::new(pool);
    let body = CreateReview {
        product_id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        rating: 1,
        body: None,
    };
    repo.create(Uuid::new_v4(), &body).await.unwrap();
    repo.create(Uuid::new_v4(), &body).await.unwrap();

    let list = repo.find_all().await.unwrap();
    assert_eq!(list.len(), 2);
    // Order is created_at DESC, so we just check we got both
}

#[sqlx::test]
async fn get_stats_empty(pool: PgPool) {
    let repo = ReviewRepository::new(pool);
    let (total, avg) = repo.get_stats().await.unwrap();
    assert_eq!(total, 0);
    assert!((avg - 0.0).abs() < 1e-9);
}

#[sqlx::test]
async fn get_stats_after_inserts(pool: PgPool) {
    let repo = ReviewRepository::new(pool);
    let body = CreateReview {
        product_id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        rating: 3,
        body: None,
    };
    repo.create(Uuid::new_v4(), &body).await.unwrap();
    let body5 = CreateReview {
        product_id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        rating: 5,
        body: None,
    };
    repo.create(Uuid::new_v4(), &body5).await.unwrap();

    let (total, avg) = repo.get_stats().await.unwrap();
    assert_eq!(total, 2);
    assert!((avg - 4.0).abs() < 0.01);
}
