use axum::routing::get_service;
use axum::Router;

use databeam::config::Config as DataConf;

use tower_http::trace::{self, TraceLayer};
use tracing::{info, Level};

mod pages;
pub use beambin_core::database;
pub use beambin_core::model;
pub use beambin_core::config;
pub use beambin_core::api;

// mimalloc
#[cfg(feature = "mimalloc")]
use mimalloc::MiMalloc;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

// ...
#[tokio::main]
async fn main() {
    let mut config = config::Config::get_config();

    let static_dir = pathbufd::PathBufD::current().extend(&[".config", "static"]);
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
            media_dir: config.media_dir.clone(),
            blocked_hosts: config.blocked_hosts.clone(),
            snowflake_server_id: config.snowflake_server_id.clone(),
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
        .merge(pages::routes(database.clone()))
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
