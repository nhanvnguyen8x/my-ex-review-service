//! Review data access. All review-related SQL lives here.

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{CreateReview, Review};

/// Repository for review persistence. No business logic, only queries.
#[derive(Clone)]
pub struct ReviewRepository {
    pool: PgPool,
}

impl ReviewRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_all(&self) -> Result<Vec<Review>, sqlx::Error> {
        sqlx::query_as::<_, Review>(
            "SELECT id, product_id, user_id, rating, body, created_at FROM reviews ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Review>, sqlx::Error> {
        sqlx::query_as::<_, Review>(
            "SELECT id, product_id, user_id, rating, body, created_at FROM reviews WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn create(&self, id: Uuid, body: &CreateReview) -> Result<Review, sqlx::Error> {
        sqlx::query(
            "INSERT INTO reviews (id, product_id, user_id, rating, body) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(id)
        .bind(body.product_id)
        .bind(body.user_id)
        .bind(body.rating)
        .bind(body.body.clone())
        .execute(&self.pool)
        .await?;

        self.find_by_id(id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)
    }

    pub async fn get_stats(&self) -> Result<(i64, f64), sqlx::Error> {
        use sqlx::Row;

        let row = sqlx::query("SELECT COUNT(*) as count FROM reviews")
            .fetch_one(&self.pool)
            .await?;
        let total: i64 = row.get::<i64, _>("count");

        let row = sqlx::query("SELECT COALESCE(AVG(rating), 0)::float8 as avg FROM reviews")
            .fetch_one(&self.pool)
            .await?;
        let avg_rating: f64 = row.get::<f64, _>("avg");

        Ok((total, avg_rating))
    }
}
