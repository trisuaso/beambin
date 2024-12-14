//! Responds to API requests
use crate::model::{
    Post, ClonePost, CreatePost, DeletePost, EditPost, EditContext, DatabaseError, PublicPost,
};

use crate::database::Database;
use axum::http::{HeaderMap, HeaderValue};
use databeam::DefaultReturn;

use axum::response::IntoResponse;
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};

pub fn routes(database: Database) -> Router {
    Router::new()
        .route("/new", post(create_request))
        .route("/clone", post(clone_request))
        // posts
        .route("/:slug", get(get_request))
        .route("/:slug/delete", post(delete_request))
        .route("/:slug/edit", post(edit_request))
        .route("/:slug/context", post(edit_post_context))
        // ...
        .with_state(database)
}

/// Create a new post (`/api/v1/new`)
async fn create_request(
    headers: HeaderMap,
    State(database): State<Database>,
    Json(props): Json<CreatePost>,
) -> Result<Json<DefaultReturn<(String, Post)>>, DatabaseError> {
    // get real ip
    let real_ip = if let Some(ref real_ip_header) = database.config.real_ip_header {
        headers
            .get(real_ip_header.to_owned())
            .unwrap_or(&HeaderValue::from_static(""))
            .to_str()
            .unwrap_or("")
            .to_string()
    } else {
        String::new()
    };

    // ...
    let res = database.create_post(props, real_ip).await;

    match res {
        Ok(p) => Ok(Json(DefaultReturn {
            success: true,
            message: String::from("Post created"),
            payload: p,
        })),
        Err(e) => Err(e),
    }
}

/// Clone an existing post (`/api/v1/clone`)
async fn clone_request(
    headers: HeaderMap,

    State(database): State<Database>,
    Json(props): Json<ClonePost>,
) -> impl IntoResponse {
    // get real ip
    let real_ip = if let Some(ref real_ip_header) = database.config.real_ip_header {
        headers
            .get(real_ip_header.to_owned())
            .unwrap_or(&HeaderValue::from_static(""))
            .to_str()
            .unwrap_or("")
            .to_string()
    } else {
        String::new()
    };

    // ...
    let res = database.clone_post(props, real_ip).await;

    match res {
        Ok(p) => Ok(Json(DefaultReturn {
            success: true,
            message: String::from("Post cloned"),
            payload: p,
        })),
        Err(e) => Err(e),
    }
}

/// Delete an existing post (`/api/v1/:slug/delete`)
async fn delete_request(
    State(database): State<Database>,
    Path(slug): Path<String>,
    Json(props): Json<DeletePost>,
) -> impl IntoResponse {
    match database.delete_post(slug, props.password).await {
        Ok(_) => Ok(Json(DefaultReturn {
            success: true,
            message: String::from("Post deleted"),
            payload: (),
        })),
        Err(e) => Err(e),
    }
}

/// Edit an existing post (`/api/v1/:slug/edit`)
async fn edit_request(
    headers: HeaderMap,
    State(database): State<Database>,
    Path(slug): Path<String>,
    Json(props): Json<EditPost>,
) -> impl IntoResponse {
    // get real ip
    let real_ip = if let Some(ref real_ip_header) = database.config.real_ip_header {
        headers
            .get(real_ip_header.to_owned())
            .unwrap_or(&HeaderValue::from_static(""))
            .to_str()
            .unwrap_or("")
            .to_string()
    } else {
        String::new()
    };

    // ...
    match database
        .edit_post(
            slug,
            real_ip,
            props.password,
            props.new_content,
            props.new_slug,
            props.new_password,
        )
        .await
    {
        Ok(_) => Ok(Json(DefaultReturn {
            success: true,
            message: String::from("Post updated"),
            payload: (),
        })),
        Err(e) => Err(e),
    }
}

/// Edit an existing post's context (`/api/v1/:slug/context`)
async fn edit_post_context(
    State(database): State<Database>,
    Path(slug): Path<String>,
    Json(props): Json<EditContext>,
) -> impl IntoResponse {
    match database
        .edit_post_context(slug, props.password, props.context)
        .await
    {
        Ok(_) => Ok(Json(DefaultReturn {
            success: true,
            message: String::from("Post updated"),
            payload: (),
        })),
        Err(e) => Err(e),
    }
}

/// Get an existing post by slug (`/api/v1/:slug`)
pub async fn get_request(
    State(database): State<Database>,
    Path(slug): Path<String>,
) -> impl IntoResponse {
    match database.get_post(slug).await {
        Ok(p) => {
            if !p.context.view_password.is_empty() {
                // cannot view from api if the post has a view password
                return Err(DatabaseError::Other);
            }

            Ok(Json(DefaultReturn {
                success: true,
                message: String::from("Post exists"),
                payload: PublicPost::from(p),
            }))
        }
        Err(e) => Err(e),
    }
}

// general
pub async fn not_found() -> impl IntoResponse {
    Json(DefaultReturn::<u16> {
        success: false,
        message: DatabaseError::NotFound.to_string(),
        payload: 404,
    })
}
