// main.rs
// This is a bare-bones starter for an API using the Axum web framework
// it has two routes: "/" - root route and "/health_check" - to return API status information
// there is a fallback route, which serves up a 404 Not Found, for routes that don't exist yet

// import dependencies
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use color_eyre::eyre::Result;
use futures::future::pending;
use std::net::SocketAddr;
use tokio::signal;
use tracing::subscriber::set_global_default;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

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

    // routes for our core API application
    let app = Router::new()
        // root route
        .route("/", get(root))
        // health_check route
        .route("/health_check", get(health_check));

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
