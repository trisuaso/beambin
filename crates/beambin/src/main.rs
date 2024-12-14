use axum::routing::get_service;
use axum::{routing::get, Router};

use databeam::config::Config as DataConf;
use rainbeam_shared::fs;

use tower_http::trace::{self, TraceLayer};
use tracing::{info, Level};

mod pages;
pub use beambin_core::database;
pub use beambin_core::model;
pub use beambin_core::config;
pub use beambin_core::api;

#[tokio::main]
async fn main() {
    let mut config = config::Config::get_config();

    let c = fs::canonicalize(".").unwrap();
    let here = c.to_str().unwrap();

    let static_dir = format!("{here}/.config/static");
    config.static_dir = static_dir.clone();

    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    // init database
    let database = database::Database::new(
        DataConf::get_config().connection,
        config::Config::get_config(),
    )
    .await;

    database.init().await;

    // ...
    let app = Router::new()
        .route("/", get(pages::homepage))
        .merge(pages::routes(database.clone()))
        .nest("/api/v1", api::routes(database.clone()))
        .nest_service(
            "/static",
            get_service(tower_http::services::ServeDir::new(&static_dir)),
        )
        .fallback(api::not_found)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        );

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", config.port))
        .await
        .unwrap();

    info!("🐝 Starting server at: http://localhost:{}!", config.port);
    axum::serve(listener, app).await.unwrap();
}
