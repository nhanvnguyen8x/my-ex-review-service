use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct Review {
    pub id: Uuid,
    pub product_id: Uuid,
    pub user_id: Uuid,
    pub rating: i32,
    pub body: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateReview {
    pub product_id: Uuid,
    pub user_id: Uuid,
    pub rating: i32,
    pub body: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ReviewResponse {
    pub id: Uuid,
    pub product_id: Uuid,
    pub user_id: Uuid,
    pub rating: i32,
    pub body: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DashboardStats {
    pub total_reviews: u64,
    pub avg_rating: f64,
}
