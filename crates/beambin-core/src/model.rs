use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use serde::{Deserialize, Serialize};
use databeam::DefaultReturn;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ViewMode {
    /// Only authenticated users can count as a post view and only once
    ///
    /// Not implemented.
    AuthenticatedOnce,
    /// Anybody can count as a post view multiple times;
    /// views are only stored in redis when using this mode
    OpenMultiple,
}

impl Default for ViewMode {
    fn default() -> Self {
        Self::OpenMultiple
    }
}

/// (timestamp, IP)
pub type IPLog = (u128, String);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Post {
    pub id: String,
    pub slug: String,
    pub content: String,
    pub password: String,
    pub date_published: u128,
    pub date_edited: u128,
    pub context: PostContext,
    pub ips: Vec<IPLog>,
}

/// Additional fields to define a [`Post`]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PostContext {
    /// Post page title
    #[serde(default)]
    pub title: String,
    /// Post page description
    #[serde(default)]
    pub description: String,
    /// Post theme color
    #[serde(default)]
    pub theme_color: String,
    /// Post favicon link
    #[serde(default)]
    pub favicon: String,
    /// Post view password (can be disabled)
    #[serde(default)]
    pub view_password: String,
    /// Post owner username
    #[serde(default)]
    pub owner: String,
    /// Post template settings
    ///
    /// * blank/no value = not a template and not using a template
    /// * `@` = this post is a template
    /// * anything else = the slug of the template post this post is derived from
    #[serde(default)]
    pub template: String,
    /// The slug of the next post in this collection
    #[serde(default)]
    pub next: String,
    /// The slug of the previous post in this collection
    #[serde(default)]
    pub previous: String,
}

impl Default for PostContext {
    fn default() -> Self {
        Self {
            title: String::new(),
            description: String::new(),
            theme_color: String::new(),
            favicon: String::new(),
            view_password: String::new(),
            owner: String::new(),
            template: String::new(),
            next: String::new(),
            previous: String::new(),
        }
    }
}

impl From<Post> for PostContext {
    /// Convert the given post into [`PostContext`] which uses the post as a template
    fn from(value: Post) -> Self {
        Self {
            template: value.slug,
            ..Default::default()
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PublicPost {
    pub slug: String,
    pub content: String,
    pub date_published: u128,
    pub date_edited: u128,
    pub context: PostContext,
}

impl From<Post> for PublicPost {
    fn from(value: Post) -> Self {
        Self {
            slug: value.slug,
            content: value.content,
            date_published: value.date_published,
            date_edited: value.date_edited,
            context: value.context,
        }
    }
}

// props

#[derive(Serialize, Deserialize, Debug)]
pub struct CreatePost {
    /// The post slug
    #[serde(default)]
    pub slug: String,
    /// The content of the post
    pub content: String,
    /// The post edit password
    #[serde(default)]
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClonePost {
    /// The slug of the post we're using as a template
    pub source: String,
    /// The post's slug
    #[serde(default)]
    pub slug: String,
    /// The post edit password
    #[serde(default)]
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeletePost {
    /// The password of the post
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EditPost {
    /// The password of the post
    pub password: String,
    /// The updated content of the post
    pub new_content: String,
    /// The updated password of the post
    #[serde(default)]
    pub new_password: String,
    /// The updated slug of the post
    #[serde(default)]
    pub new_slug: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EditContext {
    /// The password of the post
    pub password: String,
    /// The updated metadata of the post
    pub context: PostContext,
}

/// General API errors
#[derive(Debug)]
pub enum DatabaseError {
    PasswordIncorrect,
    ContentTooShort,
    ContentTooLong,
    AlreadyExists,
    NotAllowed,
    ValueError,
    NotFound,
    Banned,
    Other,
}

impl DatabaseError {
    pub fn to_string(&self) -> String {
        use DatabaseError::*;
        match self {
            PasswordIncorrect => String::from("The given password is invalid."),
            ContentTooShort => String::from("Content too short!"),
            ContentTooLong => String::from("Content too long!"),
            AlreadyExists => String::from("A post with this slug already exists."),
            NotAllowed => String::from("You are not allowed to do this!"),
            ValueError => String::from("One of the field values given is invalid!"),
            NotFound => {
                String::from("Nothing with this path exists or you do not have access to it!")
            }
            Banned => String::from("You're banned for suspected systems abuse or violating TOS."),
            _ => String::from("An unspecified error has occured"),
        }
    }
}

impl IntoResponse for DatabaseError {
    fn into_response(self) -> Response {
        use crate::model::DatabaseError::*;
        match self {
            PasswordIncorrect => (
                StatusCode::UNAUTHORIZED,
                Json(DefaultReturn::<u16> {
                    success: false,
                    message: self.to_string(),
                    payload: 401,
                }),
            )
                .into_response(),
            AlreadyExists => (
                StatusCode::BAD_REQUEST,
                Json(DefaultReturn::<u16> {
                    success: false,
                    message: self.to_string(),
                    payload: 400,
                }),
            )
                .into_response(),
            NotFound => (
                StatusCode::NOT_FOUND,
                Json(DefaultReturn::<u16> {
                    success: false,
                    message: self.to_string(),
                    payload: 404,
                }),
            )
                .into_response(),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DefaultReturn::<u16> {
                    success: false,
                    message: self.to_string(),
                    payload: 500,
                }),
            )
                .into_response(),
        }
    }
}
