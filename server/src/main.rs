use std::{sync::Arc};

use axum::{
    Json, Router, extract::State, routing::{get, post}
};
use shared::Reading;
use tokio::sync::Mutex;
#[tokio::main]
async fn main() {
    let latest = Arc::new(Mutex::new(Reading::default())); 
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/readings/latest", get(latest_handler))
        .route("/readings/submit", post(submit_handler))
        .with_state(latest.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap(); // TODO: Bind to server port defined in config.toml?

    axum::serve(listener, app).await.unwrap();
}

async fn latest_handler(State(state): State<Arc<Mutex<Reading>>>) -> String {
    let reading = state.lock().await;
    serde_json::to_string(&*reading).unwrap()
}
#[axum::debug_handler]
async fn submit_handler(State(state): State<Arc<Mutex<Reading>>>, Json(payload): Json<Reading>) {
    let mut reading = state.lock().await;
    *reading = payload;
    println!("New reading is: {} {}", reading.temperature, reading.humidity);
}
