//! Application config manager
use serde::{Deserialize, Serialize};
use std::io::Result;

use rainbeam_shared::fs;
use crate::model::ViewMode;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PostsConfig {
    /// The name of the table
    pub table_name: String,
    /// The caching prefix associated with the table
    pub prefix: String,
    // columns
    /// Mapping for the `id` column
    pub id: String,
    /// Mapping for the `slug` column
    pub slug: String,
    /// Mapping for the `password` column
    pub password: String,
    /// Mapping for the `content` column
    pub content: String,
    /// Mapping for the `date_published` column
    pub date_published: String,
    /// Mapping for the `date_edited` column
    pub date_edited: String,
    /// Mapping for the `context` column
    pub context: String,
    /// Mapping for the `ips` column
    pub ips: String,
}

impl Default for PostsConfig {
    fn default() -> Self {
        Self {
            table_name: "posts".to_string(),
            prefix: "pb.post".to_string(),
            // columns
            id: "id".to_string(),
            slug: "slug".to_string(),
            password: "password".to_string(),
            content: "content".to_string(),
            date_published: "date_published".to_string(),
            date_edited: "date_edited".to_string(),
            context: "context".to_string(),
            ips: "ips".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ViewsConfig {
    /// The name of the table
    pub table_name: String,
    /// The caching prefix associated with the table
    pub prefix: String,
}

impl Default for ViewsConfig {
    fn default() -> Self {
        Self {
            table_name: "views".to_string(),
            prefix: "pb.view".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    /// The port to serve the server on
    pub port: u16,
    /// The name of the site
    pub name: String,
    /// The description of the site
    pub description: String,
    /// The location of the static directory, should not be supplied manually as it will be overwritten with `./.config/static`
    #[serde(default)]
    pub static_dir: String,
    /// The name of the header used for reading user IP address
    pub real_ip_header: Option<String>,
    /// The origin of the public server
    ///
    /// Used in embeds and links.
    #[serde(default)]
    pub host: String,
    /// A list of external hosts that are blocked
    #[serde(default)]
    pub blocked_hosts: Vec<String>,
    /// A list of tokens that can be stored as a cookie for moderators to use to authorize moderator actions
    #[serde(default)]
    pub tokens: Vec<String>,
    // ...
    /// The slug of the server's information post
    #[serde(default)]
    pub info_post_slug: String,
    /// If posts can require a password to be viewed
    #[serde(default)]
    pub view_password: bool,
    /// If posts can have a owner username
    ///
    /// Not implemented.
    #[serde(default)]
    pub post_ownership: bool,
    /// View mode options
    #[serde(default)]
    pub view_mode: ViewMode,
    /// Posts table config
    #[serde(default)]
    pub table_posts: PostsConfig,
    /// Views table config
    #[serde(default)]
    pub table_views: ViewsConfig,
}

impl Config {
    /// Enable all options
    pub fn truthy() -> Self {
        Self {
            port: 8080,
            name: "Beambin".to_string(),
            description: String::new(),
            static_dir: "./.config".to_string(),
            host: String::new(),
            blocked_hosts: Vec::new(),
            real_ip_header: None,
            tokens: Vec::new(),
            info_post_slug: String::new(),
            view_password: true,
            post_ownership: true,
            view_mode: ViewMode::OpenMultiple,
            table_posts: PostsConfig::default(),
            table_views: ViewsConfig::default(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 8080,
            name: "Beambin".to_string(),
            description: String::new(),
            static_dir: "./.config".to_string(),
            host: String::new(),
            blocked_hosts: Vec::new(),
            real_ip_header: None,
            tokens: Vec::new(),
            info_post_slug: String::new(),
            view_password: false,
            post_ownership: false,
            view_mode: ViewMode::OpenMultiple,
            table_posts: PostsConfig::default(),
            table_views: ViewsConfig::default(),
        }
    }
}

impl Config {
    /// Read configuration file into [`Config`]
    pub fn read(contents: String) -> Self {
        toml::from_str::<Self>(&contents).unwrap()
    }

    /// Pull configuration file
    pub fn get_config() -> Self {
        let c = fs::canonicalize(".").unwrap();
        let here = c.to_str().unwrap();

        match fs::read(format!("{here}/.config/config.toml")) {
            Ok(c) => Config::read(c),
            Err(_) => {
                Self::update_config(Self::default()).expect("failed to write default config");
                Self::default()
            }
        }
    }

    /// Update configuration file
    pub fn update_config(contents: Self) -> Result<()> {
        let c = fs::canonicalize(".").unwrap();
        let here = c.to_str().unwrap();

        fs::write(
            format!("{here}/.config/config.toml"),
            toml::to_string_pretty::<Self>(&contents).unwrap(),
        )
    }
}
