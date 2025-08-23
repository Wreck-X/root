use crate::models::member::Member;
use crate::models::status_update::StatusUpdateHistory;
use crate::models::status_update::StatusUpdateStreak as Streak;
use async_graphql::{ComplexObject, Context, Object, Result};
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Default)]
pub struct StreakQueries;

#[ComplexObject]
impl StatusUpdateHistory {
    async fn member(&self, ctx: &Context<'_>) -> Result<Member> {
        let pool = ctx.data::<Arc<PgPool>>()?;
        let member = sqlx::query_as::<_, Member>("SELECT * FROM Member WHERE member_id = $1")
            .bind(self.member_id)
            .fetch_one(pool.as_ref())
            .await?;

        Ok(member)
    }
}

#[Object]
impl StreakQueries {
    async fn streak(&self, ctx: &Context<'_>, member_id: i32) -> Result<Streak> {
        let pool = ctx.data::<Arc<PgPool>>().expect("Pool must be in context.");

        Ok(sqlx::query_as::<_, Streak>(
            "SELECT current_streak, max_streak FROM StatusUpdateStreak WHERE member_id = $1",
        )
        .bind(member_id)
        .fetch_one(pool.as_ref())
        .await?)
    }

    async fn streaks(&self, ctx: &Context<'_>) -> Result<Vec<Streak>> {
        let pool = ctx.data::<Arc<PgPool>>().expect("Pool must be in context.");

        Ok(
            sqlx::query_as::<_, Streak>("SELECT * FROM StatusUpdateStreak")
                .fetch_all(pool.as_ref())
                .await?,
        )
    }

    async fn status_update_history_by_member_id(
        &self,
        ctx: &Context<'_>,
        member_id: i32,
    ) -> Result<StatusUpdateHistory> {
        let pool = ctx.data::<Arc<PgPool>>().expect("Pool must be in context.");

        Ok(sqlx::query_as::<_, StatusUpdateHistory>(
            "SELECT * FROM StatusUpdateHistory WHERE member_id = $1",
        )
        .bind(member_id)
        .fetch_one(pool.as_ref())
        .await?)
    }

    async fn status_update_history(&self, ctx: &Context<'_>) -> Result<Vec<StatusUpdateHistory>> {
        let pool = ctx.data::<Arc<PgPool>>()?;

        let rows = sqlx::query_as::<_, StatusUpdateHistory>("SELECT * FROM StatusUpdateHistory")
            .fetch_all(pool.as_ref())
            .await?;

        Ok(rows)
    }
}
