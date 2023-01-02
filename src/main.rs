// main.rs
// This is a bare-bones starter for an API using the Axum web framework.
// Database connectivity is included, the sqlx crate.
// it has four routes: "/" - root route and "/health_check" - to return API status information
// "/database_crate" - adds data to the id, date, and message fields from URL parameters
// "/database_read" - returns all data entered into the database
// "/database_update" - updates a single record by id
// "/database_delete" = deletes a single record by id
// there is a fallback route, which serves up a 404 Not Found, for routes that don't exist yet

// import dependencies
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{get, post, put},
    Router,
};
use color_eyre::eyre::Result;
use futures::future::pending;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use sqlx::FromRow;
use std::net::SocketAddr;
use tokio::signal;
use tracing::subscriber::set_global_default;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

// struct to hold data read in from the test database
#[derive(Deserialize, Serialize, Clone, Debug, FromRow)]
struct TestRecord {
    id: i32,
    date: String,
    message: String,
}

// function to handle graceful shutdown on ctl-c
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl-C graceful shutdown handler");
    };

    // configuration for graceful shutdown on Unix platforms
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    // configuration for graceful shutdown on non-Unix platforms
    #[cfg(not(unix))]
    let terminate = pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

// handler function for the "/" root route
async fn root() -> impl IntoResponse {
    (
        StatusCode::OK,
        Html("<h1>Welcome to the Axum Core API</h1><h2>Available routes:</h2><p>/ - this route, the root</p><p>/health_check - current API status</p>")
    )
}

// handler function for our "/health_check" route
async fn health_check() -> impl IntoResponse {
    (
        StatusCode::OK,
        Html("<h1>Welcome to the Axum Core API</h1><h2>Status:</h2><p>Alive, 200 OK</p>"),
    )
}

// handler function for the route which returns test data from the SQLite database
#[axum_macros::debug_handler]
async fn read_data(State(pool): State<SqlitePool>) -> impl IntoResponse {
    let record = sqlx::query_as::<_, TestRecord>("SELECT * FROM test")
        .fetch_all(&pool)
        .await
        .expect("There's been an error, could not retrieve the records from the database.");

    (StatusCode::OK, Json(record)).into_response()
}

// handler function for the route which adds some data to the SQLite database
// data is hardcoded for the time being
#[axum_macros::debug_handler]
async fn create_data(
    State(pool): State<SqlitePool>,
    Query(params): Query<TestRecord>,
) -> impl IntoResponse {
    let _result = sqlx::query("INSERT INTO test (id, date, message) VALUES ($1, $2, $3)")
        .bind(params.id)
        .bind(params.date)
        .bind(params.message)
        .execute(&pool)
        .await
        .expect("Error writing to database, could not write new values.");
    (
        StatusCode::OK,
        Html("<h1>Data added...check /database_read for results</h1>"),
    )
}

#[axum_macros::debug_handler]
async fn update_data(
    State(pool): State<SqlitePool>,
    Query(params): Query<TestRecord>,
) -> impl IntoResponse {
    let _result = sqlx::query("UPDATE test SET message=$3 where id=$1")
        .bind(params.id)
        .bind(params.message)
        .execute(&pool)
        .await
        .expect("Failed to update the record.");
    (
        StatusCode::OK,
        Html("<h1>Data updated...check /database_check for results</h1>"),
    )
}

#[axum_macros::debug_handler]
async fn delete_data(
    State(pool): State<SqlitePool>,
    Query(params): Query<TestRecord>,
) -> impl IntoResponse {
    let _result = sqlx::query("DELETE FROM test WHERE id = $1")
        .bind(params.id)
        .execute(&pool)
        .await
        .expect("Error deleting the record from the database.");
    (
        StatusCode::OK,
        Html("<h1>Deleted record...check /database_check to confirm."),
    )
}

#[axum_macros::debug_handler]
async fn search_data(
    State(pool): State<SqlitePool>,
    Query(params): Query<TestRecord>
) -> impl IntoResponse {
    let record = sqlx::query_as::<_, TestRecord>("SELECT * FROM test WHERE id = $1 ")
        .bind(params.id)
        .fetch_one(&pool)
        .await
        .expect("There's been an error, could not retrieve the record from the database.");

    (StatusCode::OK, Json(record)).into_response()
}

// handler function for non existent routes, returns a 404 Not Found
async fn not_found_404() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Html("<h1>Nothing here by that name...yet.</h1>"),
    )
}

// main application
#[tokio::main]
async fn main() -> Result<()> {
    // initialize color_eyre for nice looking error messages
    color_eyre::install()?;

    // initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    set_global_default(subscriber)?;

    // SQLite database pool setup
    let db_connection_str = "sqlite://db/test.db";
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(db_connection_str)
        .await?;

    // routes for our core API application, store the database connection pool in state
    let app = Router::new()
        // root route
        .route("/", get(root))
        // health_check route
        .route("/health_check", get(health_check))
        .route("/database_read", get(read_data))
        .route("/database_create", post(create_data))
        .route("/database_update", put(update_data))
        .route("/database_delete", post(delete_data))
        .route("/database_search", get(search_data))
        .with_state(pool);

    let app = app.fallback(not_found_404);

    // spin up and listen on port 127.0.0.1:3000
    let port = 3000;
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!("listening on port: {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}
