use axum::routing::get_service;
use axum::Router;

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
    let auth_database = beambin_core::AuthDatabase::new(
        DataConf::get_config().connection, // pull connection config from config file
        beambin_core::AuthServerOptions {
            captcha: config.captcha.clone(),
            // registration_enabled: config.registration_enabled,
            registration_enabled: false,
            real_ip_header: config.real_ip_header.clone(),
            static_dir: config.static_dir.clone(),
            host: config.host.clone(),
            citrus_id: String::new(),
            blocked_hosts: config.blocked_hosts.clone(),
            // secure: config.secure.clone(),
            secure: true,
        },
    )
    .await;
    auth_database.init().await;

    let database = database::Database::new(
        DataConf::get_config().connection,
        auth_database.clone(),
        config.clone(),
    )
    .await;
    database.init().await;

    // ...
    let app = Router::new()
        .nest("/", pages::routes(database.clone()))
        .nest("/api/v1/posts", api::posts::routes(database.clone()))
        .nest("/api/v0/util", api::util::routes(database.clone()))
        .nest("/api/v0/auth", beambin_core::authapi::routes(auth_database))
        .nest_service(
            "/static",
            get_service(tower_http::services::ServeDir::new(&static_dir)),
        )
        .fallback(api::posts::not_found)
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
