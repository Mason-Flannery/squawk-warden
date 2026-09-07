use sqlx::SqlitePool;
use sqlx_sqlite::SqliteConnectOptions;
use std::{env, str::FromStr};

pub async fn init() -> SqlitePool {
    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL is not defined in the environment");

    let options = SqliteConnectOptions::from_str(&database_url)
        .unwrap()
        .create_if_missing(true);

    let db: SqlitePool = sqlx::SqlitePool::connect_with(options)
        .await
        .expect("Unable to connect to the database");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS readings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            temperature REAL NOT NULL,
            humidity REAL NOT NULL
        )
        "#,
    )
    .execute(&db)
    .await
    .expect("We should be able to create a table");

    db
}

pub async fn get_latest(db: &SqlitePool) -> Result<Option<Reading>, sqlx::Error> {
    let reading = sqlx::query_as(
        "SELECT id, timestamp, temperature, humidity 
            FROM readings 
            ORDER BY id DESC 
            LIMIT 1;",
    )
    .fetch_optional(db)
    .await
    .expect("We should be able to create a table");

    Ok(reading)
}

pub async fn new_reading(
    db: &SqlitePool,
    temperature: f32,
    humidity: f32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO readings (temperature, humidity)
        VALUES (?, ?)
        "#,
    )
    .bind(temperature)
    .bind(humidity)
    .execute(db)
    .await?;

    Ok(())
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct Reading {
    pub id: i64,
    pub timestamp: String, // or sqlx::types::time/chrono types if you prefer
    pub temperature: f32,
    pub humidity: f32,
}
