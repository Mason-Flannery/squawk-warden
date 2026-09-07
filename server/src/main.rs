pub mod db;
use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use dotenvy::dotenv;
use shared::Reading;
use sqlx::SqlitePool;

#[tokio::main]
async fn main() {
    let _ = dotenv(); // Load environment variables 

    let latest = db::init().await;
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/readings/latest", get(latest_handler))
        .route("/readings/submit", post(submit_handler))
        .with_state(latest);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap(); // TODO: Bind to server port defined in config.toml?

    axum::serve(listener, app).await.unwrap();
}

async fn latest_handler(State(state): State<SqlitePool>) -> Json<Reading> {
    if let Ok(Some(reading)) = db::get_latest(&state).await {
        Json(Reading {
            temperature: reading.temperature,
            humidity: reading.humidity,
        })
    } else {
        // Lazy approach, need to propagate errors once frontend is skeletonized
        Json(Reading {
            temperature: -1.0,
            humidity: -1.0,
        })
    }
}
#[axum::debug_handler]
async fn submit_handler(State(state): State<SqlitePool>, Json(payload): Json<Reading>) {
    if let Err(result) = db::new_reading(&state, payload.temperature, payload.humidity).await {
        println!("An error occurred writing to disk: {}", result);
    } else {
        println!(
            "New reading is: {} {}",
            payload.temperature, payload.humidity
        );
    }
}
