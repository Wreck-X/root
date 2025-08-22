use sqlx::PgPool;
use std::fs;
use std::path::Path;

pub async fn seed_database(pool: &PgPool) {
    let sql_path = Path::new("src/database_seeder/seed.sql");
    let seed_sql = fs::read_to_string(sql_path).expect("Failed to read seed.sql file");

    let statements: Vec<&str> = seed_sql
        .split(';')
        .filter(|stmt| !stmt.trim().is_empty())
        .collect();

    for statement in statements {
        sqlx::query(statement)
            .execute(pool)
            .await
            .expect("Failed to execute seed statement");
    }
}
