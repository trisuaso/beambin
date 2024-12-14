use askama_axum::Template;
use axum::{
    extract::{Path, State, Query},
    response::{Html, Json, IntoResponse},
    routing::{get, post},
    Router,
};

use axum_extra::extract::CookieJar;
use serde::{Serialize, Deserialize};

use beambin_core::{
    database::Database,
    model::{DatabaseError, Post},
};
use rainbeam_shared::ui::render_markdown as md;

pub fn routes(database: Database) -> Router {
    Router::new()
        .route("/:url/edit/config", get(config_editor_request))
        .route("/:url/edit", get(editor_request))
        .route("/:url", get(view_post_request))
        .route("/api/v0/render", post(render_markdown))
        // ...
        .with_state(database)
}

#[derive(Template)]
#[template(path = "homepage.html")]
struct HomepageTemplate {}

pub async fn homepage() -> impl IntoResponse {
    Html(HomepageTemplate {}.render().unwrap())
}

#[derive(Template)]
#[template(path = "post/view.html")]
struct PostViewTemplate {
    post: Post,
    rendered: String,
    title: String,
    views: i32,
    head_stuff: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PostViewQuery {
    #[serde(default)]
    view_password: String,
}

#[derive(Template)]
#[template(path = "post/password_prompt.html")]
struct PostPasswordTemplate {
    post: Post,
}

#[derive(Template)]
#[template(path = "error.html")]
struct ErrorViewTemplate {
    error: String,
}

pub async fn view_post_request(
    Path(slug): Path<String>,
    State(database): State<Database>,
    Query(query_params): Query<PostViewQuery>,
) -> impl IntoResponse {
    match database.get_post(slug).await {
        Ok(p) => {
            // check for view password
            if database.config.view_password == true {
                match query_params.view_password.is_empty() {
                    false => {
                        if !p.context.view_password.is_empty()
                            && (query_params.view_password != p.context.view_password)
                        {
                            return Html(PostPasswordTemplate { post: p }.render().unwrap());
                        }
                    }
                    true => {
                        if !p.context.view_password.is_empty() {
                            return Html(PostPasswordTemplate { post: p }.render().unwrap());
                        }
                    }
                }
            }

            // push view
            // we could not support paste views by just.. not doing this
            if let Err(e) = database.incr_views_by_slug(p.slug.clone()).await {
                return Html(
                    ErrorViewTemplate {
                        error: e.to_string(),
                    }
                    .render()
                    .unwrap(),
                );
            }

            // ...
            let rendered = md(&p.content.clone());
            Html(
                PostViewTemplate {
                    post: p.clone(),
                    rendered,
                    title: match p.context.title.is_empty() {
                        true => p.slug.clone(),
                        false => p.context.title,
                    },
                    views: database.get_views_by_slug(p.slug).await,
                    head_stuff: format!(
                        "<meta property=\"og:description\" content=\"{}\" />
                        <meta name=\"theme-color\" content=\"{}\" />
                        <link rel=\"icon\" href=\"{}\" />",
                        if p.context.description.is_empty() {
                            // paste preview text
                            p.content
                                .chars()
                                .take(100)
                                .collect::<String>()
                                .replace("\"", "'")
                        } else {
                            p.context.description
                        },
                        if p.context.theme_color.is_empty() {
                            "#6ee7b7"
                        } else {
                            &p.context.theme_color
                        },
                        if p.context.favicon.is_empty() {
                            "/static/favicon.svg"
                        } else {
                            &p.context.favicon
                        }
                    ),
                }
                .render()
                .unwrap(),
            )
        }
        Err(e) => Html(
            ErrorViewTemplate {
                error: e.to_string(),
            }
            .render()
            .unwrap(),
        ),
    }
}

#[derive(Template)]
#[template(path = "post/editor.html")]
struct EditorTemplate {
    post: Post,
    passwordless: bool,
    is_admin: bool,
}

pub async fn editor_request(
    jar: CookieJar,
    Path(slug): Path<String>,
    State(database): State<Database>,
    Query(query_params): Query<PostViewQuery>,
) -> impl IntoResponse {
    // get auth token
    let auth_token = match jar.get("__Secure-Token") {
        Some(c) => c.value_trimmed().to_string(),
        None => String::new(),
    };

    // ...
    match database.get_post(slug).await {
        Ok(p) => {
            // check for view password
            if database.config.view_password == true {
                match query_params.view_password.is_empty() {
                    false => {
                        if !p.context.view_password.is_empty()
                            && (query_params.view_password != p.context.view_password)
                        {
                            return Html(PostPasswordTemplate { post: p }.render().unwrap());
                        }
                    }
                    true => {
                        if !p.context.view_password.is_empty() {
                            return Html(PostPasswordTemplate { post: p }.render().unwrap());
                        }
                    }
                }
            }

            // ...
            Html(
                EditorTemplate {
                    post: p,
                    passwordless: false,
                    is_admin: database.config.tokens.contains(&auth_token),
                }
                .render()
                .unwrap(),
            )
        }
        Err(e) => Html(
            ErrorViewTemplate {
                error: e.to_string(),
            }
            .render()
            .unwrap(),
        ),
    }
}

#[derive(Template)]
#[template(path = "post/context.html")]
struct ConfigEditorTemplate {
    post: Post,
    post_context: String,
    auth_token: String,
    passwordless: bool,
    is_admin: bool,
}

pub async fn config_editor_request(
    jar: CookieJar,
    Path(slug): Path<String>,
    State(database): State<Database>,
    Query(query_params): Query<PostViewQuery>,
) -> impl IntoResponse {
    // get auth token
    let auth_token = match jar.get("__Secure-Token") {
        Some(c) => c.value_trimmed().to_string(),
        None => String::new(),
    };

    // ...
    match database.get_post(slug).await {
        Ok(p) => {
            // check for view password
            if database.config.view_password == true {
                match query_params.view_password.is_empty() {
                    false => {
                        if !p.context.view_password.is_empty()
                            && (query_params.view_password != p.context.view_password)
                        {
                            return Html(PostPasswordTemplate { post: p }.render().unwrap());
                        }
                    }
                    true => {
                        if !p.context.view_password.is_empty() {
                            return Html(PostPasswordTemplate { post: p }.render().unwrap());
                        }
                    }
                }
            }

            // ...
            Html(
                ConfigEditorTemplate {
                    post: p.clone(),
                    post_context: match serde_json::to_string(&p.context) {
                        Ok(m) => m,
                        Err(_) => {
                            return Html(
                                ErrorViewTemplate {
                                    error: DatabaseError::Other.to_string(),
                                }
                                .render()
                                .unwrap(),
                            )
                        }
                    },
                    is_admin: database.config.tokens.contains(&auth_token),
                    auth_token,
                    passwordless: false,
                }
                .render()
                .unwrap(),
            )
        }
        Err(e) => Html(
            ErrorViewTemplate {
                error: e.to_string(),
            }
            .render()
            .unwrap(),
        ),
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct RenderMarkdown {
    pub content: String,
}

/// Render markdown body
async fn render_markdown(Json(req): Json<RenderMarkdown>) -> Result<String, ()> {
    Ok(md(&req.content.clone()))
}
