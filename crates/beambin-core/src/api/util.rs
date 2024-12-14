use crate::database::Database;
use axum::{
    body::Body,
    extract::{Query, State},
    response::IntoResponse,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Read};

pub fn routes(database: Database) -> Router {
    Router::new()
        .route("/ext/image", get(external_image_request))
        // ...
        .with_state(database.clone())
}

pub fn read_image(static_dir: String, image: String) -> Vec<u8> {
    let mut bytes = Vec::new();

    for byte in File::open(format!("{static_dir}/images/{image}"))
        .unwrap()
        .bytes()
    {
        bytes.push(byte.unwrap())
    }

    bytes
}

#[derive(Serialize, Deserialize)]
pub struct ExternalImageQuery {
    pub img: String,
}

/// Proxy an external image
pub async fn external_image_request(
    Query(props): Query<ExternalImageQuery>,
    State(database): State<Database>,
) -> impl IntoResponse {
    let image_url = &props.img;

    if image_url.starts_with(&database.config.host) {
        return (
            [("Content-Type", "image/svg+xml")],
            Body::from(read_image(
                database.config.static_dir,
                "default.svg".to_string(),
            )),
        );
    }

    for host in database.config.blocked_hosts {
        if image_url.starts_with(&host) {
            return (
                [("Content-Type", "image/svg+xml")],
                Body::from(read_image(
                    database.config.static_dir,
                    "default.svg".to_string(),
                )),
            );
        }
    }

    // get profile image
    if image_url.is_empty() {
        return (
            [("Content-Type", "image/svg+xml")],
            Body::from(read_image(
                database.config.static_dir,
                "default.svg".to_string(),
            )),
        );
    }

    let guessed_mime = mime_guess::from_path(image_url)
        .first_raw()
        .unwrap_or("application/octet-stream");

    match database.http.get(image_url).send().await {
        Ok(stream) => {
            if let Some(ct) = stream.headers().get("Content-Type") {
                let bad_ct = vec!["text/html", "text/plain"];
                if bad_ct.contains(&ct.to_str().unwrap()) {
                    // if we got html, return default banner (likely an error page)
                    return (
                        [("Content-Type", "image/svg+xml")],
                        Body::from(read_image(
                            database.config.static_dir,
                            "default.svg".to_string(),
                        )),
                    );
                }
            }

            (
                [(
                    "Content-Type",
                    if guessed_mime == "text/html" {
                        "text/plain"
                    } else {
                        guessed_mime
                    },
                )],
                Body::from_stream(stream.bytes_stream()),
            )
        }
        Err(_) => (
            [("Content-Type", "image/svg+xml")],
            Body::from(read_image(
                database.config.static_dir,
                "default.svg".to_string(),
            )),
        ),
    }
}
