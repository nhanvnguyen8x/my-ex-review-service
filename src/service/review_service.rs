//! Review business logic. Handles validation, orchestration, and mapping to API types.

use uuid::Uuid;

use crate::models::{CreateReview, DashboardStats, Review, ReviewResponse};
use crate::repository::ReviewRepository;

/// Application service for reviews and review-derived stats.
#[derive(Clone)]
pub struct ReviewService {
    repo: ReviewRepository,
}

impl ReviewService {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            repo: ReviewRepository::new(pool),
        }
    }

    pub async fn list_reviews(&self) -> Result<Vec<ReviewResponse>, String> {
        let rows = self.repo.find_all().await.map_err(|e| e.to_string())?;
        Ok(rows.into_iter().map(review_to_response).collect())
    }

    pub async fn get_review(&self, id: Uuid) -> Result<ReviewResponse, String> {
        let r = self.repo.find_by_id(id).await.map_err(|e| e.to_string())?;
        r.map(review_to_response)
            .ok_or_else(|| "Not found".to_string())
    }

    pub async fn create_review(&self, body: CreateReview) -> Result<ReviewResponse, String> {
        let id = Uuid::new_v4();
        let r = self.repo.create(id, &body).await.map_err(|e| e.to_string())?;
        Ok(review_to_response(r))
    }

    pub async fn get_dashboard_stats(&self) -> Result<DashboardStats, String> {
        let (total, avg_rating) = self.repo.get_stats().await.map_err(|e| e.to_string())?;
        Ok(DashboardStats {
            total_reviews: total as u64,
            avg_rating,
        })
    }
}

fn review_to_response(r: Review) -> ReviewResponse {
    ReviewResponse {
        id: r.id,
        product_id: r.product_id,
        user_id: r.user_id,
        rating: r.rating,
        body: r.body,
        created_at: r.created_at,
    }
}
