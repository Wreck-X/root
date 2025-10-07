use async_graphql::SimpleObject;
use chrono::NaiveDate;
use sqlx::FromRow;

#[derive(SimpleObject, FromRow)]
pub struct StatusUpdateRecord {
    pub update_id: i32,
    pub member_id: i32,
    pub date: NaiveDate,
    pub is_sent: bool,
}

#[derive(SimpleObject, FromRow)]
pub struct StatusUpdateStreakRecord {
    pub current_streak: Option<i64>,
    pub max_streak: Option<i64>,
}
