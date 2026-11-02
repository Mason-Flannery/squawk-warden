mod db;
use axum::{
    Json, Router, extract::{Query, State}, http::StatusCode, response::IntoResponse, routing::{get, post},
};
use dotenvy::dotenv;
use serde::Deserialize;
use shared::{DoorAction, DoorCommand, Reading};
use sqlx::SqlitePool;
use tower_http::services::ServeDir;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    let _ = dotenv(); // Load environment variables 
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any);
    let door = Arc::new(Mutex::new(DoorCommand::default()));

    let latest = db::init().await;
    let readings = Router::new()
        .route("/readings/latest", get(latest_handler))
        .route("/readings/submit", post(submit_handler))
        .route("/readings/history", get(history_handler))
        .with_state(latest);

    let commands = Router::new()
    .route("/commands/door", get(get_door).post(set_door))
    .route("/commands/door/ack", post(ack_door))
    .with_state(door);

    let app = Router::new()
    .route("/", get(|| async { "Hello, World!" }))
    .merge(readings)
    .merge(commands)
    .fallback_service(ServeDir::new("../frontend/dist"))
    .layer(cors);;


    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap(); // TODO: Bind to server port defined in config.toml?

    axum::serve(listener, app).await.unwrap();
}

async fn latest_handler(State(state): State<SqlitePool>) -> impl IntoResponse {
    match db::get_latest(&state).await {
        Ok(Some(reading)) => Json(reading).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, "no readings yet").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
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

#[derive(Deserialize)]
pub struct HistoryParams {
    limit: Option<i64>,
}
async fn history_handler(
    State(db): State<SqlitePool>,
    Query(params): Query<HistoryParams>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(200).clamp(1, 5000);
    match db::get_history(&db, limit).await {
        Ok(readings) => (StatusCode::OK, Json(readings)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn get_door(State(door): State<DoorCommand>) -> impl IntoResponse {
    Json(door.lock().unwrap().clone())
}

async fn set_door(State(door): State<DoorCommand>, Json(action): Json<DoorAction>,) -> impl IntoResponse {
    let mut cmd = door.lock().unwrap();

    let mut id = rand::random::<u32>();
    while id == 0 || id == cmd.id { // Make SURE we don't collide
        id = rand::random::<u32>();
    }

    *cmd = DoorCommand {id, action};
    Json(cmd.clone())
}

async fn ack_door(State(door): State<DoorCommand>, Json(ack): Json<CommandAck>) -> StatusCode {
    let mut cmd = door.lock().unwrap();
    if cmd.id == ack.id {
        *cmd = DoorCommand::default();
    }
    StatusCode::OK
}