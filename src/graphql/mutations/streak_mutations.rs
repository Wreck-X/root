use std::sync::Arc;

use async_graphql::{Context, Object, Result};
use sqlx::PgPool;

use crate::models::status_update::{StatusUpdateStreak as Streak, StreakInput};
use chrono_tz::Asia::Kolkata;

#[derive(Default)]
pub struct StreakMutations;

#[Object]
impl StreakMutations {
    #[graphql(name = "incrementStreak")]
    async fn increment_streak(&self, ctx: &Context<'_>, input: StreakInput) -> Result<Streak> {
        let pool = ctx.data::<Arc<PgPool>>().expect("Pool must be in context.");

        let query = sqlx::query_as::<_, Streak>(
            "
        INSERT INTO StatusUpdateStreak (member_id, current_streak, max_streak)
        VALUES ($1, 1, 1)
        ON CONFLICT (member_id) DO UPDATE SET 
            current_streak = CASE
                WHEN StatusUpdateStreak.current_streak >= 0 THEN StatusUpdateStreak.current_streak + 1
                ELSE 1
            END,
            max_streak = GREATEST(StatusUpdateStreak.max_streak, StatusUpdateStreak.current_streak + 1)
        RETURNING *",
        )
        .bind(input.member_id);

        let updated_streak = query.fetch_one(pool.as_ref()).await?;

        update_status_history(pool.as_ref(), input.member_id).await?;

        Ok(updated_streak)
    }

    async fn reset_streak(&self, ctx: &Context<'_>, input: StreakInput) -> Result<Streak> {
        let pool = ctx.data::<Arc<PgPool>>().expect("Pool must be in context.");

        let query = sqlx::query_as::<_, Streak>(
            "
        INSERT INTO StatusUpdateStreak (member_id, current_streak, max_streak)
        VALUES ($1, 0, 0)
        ON CONFLICT (member_id) DO UPDATE
            SET current_streak = CASE
                WHEN StatusUpdateStreak.current_streak > 0 THEN 0
                ELSE StatusUpdateStreak.current_streak - 1 
            END
        RETURNING *",
        )
        .bind(input.member_id);

        let updated_streak = query.fetch_one(pool.as_ref()).await?;
        Ok(updated_streak)
    }
}

async fn update_status_history(pool: &PgPool, member_id: i32) -> Result<()> {
    #[allow(deprecated)]
    let yesterday = chrono::Utc::now()
        .with_timezone(&Kolkata)
        .date()
        .naive_local()
        - chrono::Duration::days(1);

    sqlx::query(
        "
        UPDATE StatusUpdateHistory 
        SET is_updated = TRUE
        WHERE member_id = $1
        AND date = $2
        ",
    )
    .bind(member_id)
    .bind(yesterday)
    .execute(pool)
    .await?;

    Ok(())
}
