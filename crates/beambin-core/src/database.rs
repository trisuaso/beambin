use crate::model::{CreatePost, DatabaseError, Post, PostContext, ClonePost, ViewMode};
use crate::config::Config;

use authbeam::model::Profile;
use reqwest::Client as HttpClient;

use databeam::utility;
use databeam::query as sqlquery;

pub type Result<T> = std::result::Result<T, DatabaseError>;

/// Database connector
#[derive(Clone)]
pub struct Database {
    pub base: databeam::StarterDatabase,
    pub auth: authbeam::Database,
    pub config: Config,
    pub http: HttpClient,
}

impl Database {
    /// Create a new [`Database`]
    pub async fn new(
        database_options: databeam::DatabaseOpts,
        auth: authbeam::Database,
        config: Config,
    ) -> Self {
        Self {
            base: databeam::StarterDatabase::new(database_options).await,
            auth,
            config,
            http: HttpClient::new(),
        }
    }

    /// Init database
    pub async fn init(&self) {
        // create tables
        let c = &self.base.db.client;

        let _ = sqlquery(&format!(
            "CREATE TABLE IF NOT EXISTS \"{}\" (
                {} TEXT,
                {} TEXT,
                {} TEXT,
                {} TEXT,
                {} TEXT,
                {} TEXT,
                {} TEXT,
                {} TEXT
            )",
            // table
            self.config.table_posts.table_name,
            // columns
            self.config.table_posts.id,
            self.config.table_posts.slug,
            self.config.table_posts.password,
            self.config.table_posts.content,
            self.config.table_posts.date_published,
            self.config.table_posts.date_edited,
            self.config.table_posts.context,
            self.config.table_posts.ips
        ))
        .execute(c)
        .await;

        if self.config.view_mode == ViewMode::AuthenticatedOnce {
            // create table to track views
            let _ = sqlquery(&format!(
                "CREATE TABLE IF NOT EXISTS \"{}\" (
                    slug     TEXT,
                    id       TEXT
                )",
                self.config.table_views.table_name
            ))
            .execute(c)
            .await;
        }
    }

    // ...

    /// Get an existing post
    ///
    /// # Arguments
    /// * `slug` - [`String`] of the posts's `slug` field
    pub async fn get_post(&self, mut slug: String) -> Result<Post> {
        slug = idna::punycode::encode_str(&slug).unwrap().to_lowercase();

        if slug.ends_with("-") {
            slug.pop();
        }

        // check in cache
        match self
            .base
            .cachedb
            .get(format!("{}:{}", self.config.table_posts.prefix, slug))
            .await
        {
            Some(c) => return Ok(serde_json::from_str::<Post>(c.as_str()).unwrap()),
            None => (),
        };

        // pull from database
        let query: String = if (self.base.db.r#type == "sqlite") | (self.base.db.r#type == "mysql")
        {
            "SELECT * FROM \":t\" WHERE \":slug\" = ?"
        } else {
            "SELECT * FROM \":t\" WHERE \":slug\" = $1"
        }
        .to_string()
        .replace(":t", &self.config.table_posts.table_name)
        .replace(":slug", &self.config.table_posts.slug);

        let c = &self.base.db.client;
        let res = match sqlquery(&query)
            .bind::<&String>(&slug.to_lowercase())
            .fetch_one(c)
            .await
        {
            Ok(p) => self.base.textify_row(p, Vec::new()).0,
            Err(_) => return Err(DatabaseError::NotFound),
        };

        // return
        let post = Post {
            id: res.get(&self.config.table_posts.id).unwrap().to_string(),
            slug: res.get(&self.config.table_posts.slug).unwrap().to_string(),
            password: res
                .get(&self.config.table_posts.password)
                .unwrap()
                .to_string(),
            content: res
                .get(&self.config.table_posts.content)
                .unwrap()
                .to_string(),
            date_published: res
                .get(&self.config.table_posts.date_published)
                .unwrap()
                .parse::<u128>()
                .unwrap(),
            date_edited: res
                .get(&self.config.table_posts.date_edited)
                .unwrap()
                .parse::<u128>()
                .unwrap(),
            context: match serde_json::from_str(res.get(&self.config.table_posts.context).unwrap())
            {
                Ok(m) => m,
                Err(_) => return Err(DatabaseError::ValueError),
            },
            ips: match serde_json::from_str(res.get(&self.config.table_posts.ips).unwrap()) {
                Ok(m) => m,
                Err(_) => return Err(DatabaseError::ValueError),
            },
        };

        // store in cache
        self.base
            .cachedb
            .set(
                format!("{}:{}", self.config.table_posts.prefix, slug),
                serde_json::to_string::<Post>(&post).unwrap(),
            )
            .await;

        // return
        Ok(post)
    }

    /// Create a new post
    ///
    /// # Arguments
    /// * `props` - [`PostCreate`]
    /// * `ip` - the IP address where this post was created
    ///
    /// # Returns
    /// * Result containing a tuple with the unhashed edit password and the post
    pub async fn create_post(&self, mut props: CreatePost, ip: String) -> Result<(String, Post)> {
        props.slug = idna::punycode::encode_str(&props.slug)
            .unwrap()
            .to_lowercase();

        if props.slug.ends_with("-") {
            props.slug.pop();
        }

        // make sure post doesn't already exist
        if let Ok(_) = self.get_post(props.slug.clone()).await {
            return Err(DatabaseError::AlreadyExists);
        }

        // create slug if not supplied
        if props.slug.is_empty() {
            props.slug = utility::random_id().chars().take(10).collect();
        }

        // create random password if not supplied
        if props.password.is_empty() {
            props.password = utility::random_id().chars().take(10).collect();
        }

        // check lengths
        if (props.slug.len() > 250) | (props.slug.len() < 3) {
            return Err(DatabaseError::ValueError);
        }

        if (props.content.len() > 200_000) | (props.content.len() < 1) {
            return Err(DatabaseError::ValueError);
        }

        // (characters used)
        let regex = regex::RegexBuilder::new("^[\\w\\_\\-\\.\\!\\p{Extended_Pictographic}]+$")
            .multi_line(true)
            .build()
            .unwrap();

        if regex.captures(&props.slug).iter().len() < 1 {
            return Err(DatabaseError::ValueError);
        }

        // ...
        let post = Post {
            id: utility::random_id(),
            slug: props.slug,
            content: props.content,
            password: utility::hash(props.password.clone()),
            date_published: utility::unix_epoch_timestamp(),
            date_edited: utility::unix_epoch_timestamp(),
            context: PostContext::default(),
            ips: vec![(utility::unix_epoch_timestamp(), ip)],
        };

        // create post
        let query: String = if (self.base.db.r#type == "sqlite") | (self.base.db.r#type == "mysql")
        {
            "INSERT INTO \":t\" VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        } else {
            "INSERT INTO \":t\" VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
        }
        .to_string()
        .replace(":t", &self.config.table_posts.table_name);

        let c = &self.base.db.client;
        match sqlquery(&query)
            .bind::<&String>(&post.id)
            .bind::<&String>(&post.slug)
            .bind::<&String>(&post.password)
            .bind::<&String>(&post.content)
            .bind::<&String>(&post.date_published.to_string())
            .bind::<&String>(&post.date_edited.to_string())
            .bind::<&String>(match serde_json::to_string(&post.context) {
                Ok(ref s) => s,
                Err(_) => return Err(DatabaseError::ValueError),
            })
            .bind::<&String>(match serde_json::to_string(&post.ips) {
                Ok(ref s) => s,
                Err(_) => return Err(DatabaseError::ValueError),
            })
            .execute(c)
            .await
        {
            Ok(_) => return Ok((props.password, post)),
            Err(_) => return Err(DatabaseError::Other),
        };
    }

    /// Use an existing post as a template
    ///
    /// # Arguments
    /// * `props` - [`PostClone`]
    /// * `ip` - the IP address where this post was created
    ///
    /// # Returns
    /// * Result containing a tuple with the unhashed edit password and the post
    pub async fn clone_post(&self, mut props: ClonePost, ip: String) -> Result<(String, Post)> {
        props.slug = idna::punycode::encode_str(&props.slug)
            .unwrap()
            .to_lowercase();

        if props.slug.ends_with("-") {
            props.slug.pop();
        }

        // make sure post doesn't already exist
        if let Ok(_) = self.get_post(props.slug.clone()).await {
            return Err(DatabaseError::AlreadyExists);
        }

        // make sure post source exists
        let source = match self.get_post(props.source).await {
            Ok(p) => p,
            Err(e) => return Err(e),
        };

        // create slug if not supplied
        if props.slug.is_empty() {
            props.slug = utility::random_id().chars().take(10).collect();
        }

        // create random password if not supplied
        if props.password.is_empty() {
            props.password = utility::random_id().chars().take(10).collect();
        }

        // check lengths
        if (props.slug.len() > 250) | (props.slug.len() < 3) {
            return Err(DatabaseError::ValueError);
        }

        // (characters used)
        let regex = regex::RegexBuilder::new("^[\\w\\_\\-\\.\\!\\p{Extended_Pictographic}]+$")
            .multi_line(true)
            .build()
            .unwrap();

        if regex.captures(&props.slug).iter().len() < 1 {
            return Err(DatabaseError::ValueError);
        }

        // ...
        let source_c = source.clone();
        let post = Post {
            id: utility::random_id(),
            slug: props.slug,
            content: source.content,
            password: utility::hash(props.password.clone()),
            date_published: utility::unix_epoch_timestamp(),
            date_edited: utility::unix_epoch_timestamp(),
            context: PostContext::from(source_c), // use other post as a template
            ips: vec![(utility::unix_epoch_timestamp(), ip)],
        };

        // create post
        let query: String = if (self.base.db.r#type == "sqlite") | (self.base.db.r#type == "mysql")
        {
            "INSERT INTO \":t\" VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        } else {
            "INSERT INTO \":t\" VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
        }
        .to_string()
        .replace(":t", &self.config.table_posts.table_name);

        let c = &self.base.db.client;
        match sqlquery(&query)
            .bind::<&String>(&post.id)
            .bind::<&String>(&post.slug)
            .bind::<&String>(&post.password)
            .bind::<&String>(&post.content)
            .bind::<&String>(&post.date_published.to_string())
            .bind::<&String>(&post.date_edited.to_string())
            .bind::<&String>(match serde_json::to_string(&post.context) {
                Ok(ref s) => s,
                Err(_) => return Err(DatabaseError::ValueError),
            })
            .bind::<&String>(match serde_json::to_string(&post.ips) {
                Ok(ref s) => s,
                Err(_) => return Err(DatabaseError::ValueError),
            })
            .execute(c)
            .await
        {
            Ok(_) => return Ok((props.password, post)),
            Err(_) => return Err(DatabaseError::Other),
        };
    }

    /// Delete an existing post
    ///
    /// # Arguments
    /// * `slug` - the post to delete
    /// * `password` - the post's edit password
    pub async fn delete_post(
        &self,
        mut slug: String,
        password: String,
        user: Option<Box<Profile>>,
    ) -> Result<()> {
        slug = idna::punycode::encode_str(&slug).unwrap().to_lowercase();

        if slug.ends_with("-") {
            slug.pop();
        }

        // get post
        let existing = match self.get_post(slug.clone()).await {
            Ok(p) => p,
            Err(err) => return Err(err),
        };

        // check password
        let mut skip_password_check = false;

        if let Some(ref ua) = user {
            // check permission
            let group = match self.auth.get_group_by_id(ua.group).await {
                Ok(g) => g,
                Err(_) => return Err(DatabaseError::Other),
            };

            if !group
                .permissions
                .contains(&authbeam::model::Permission::Manager)
            {
                return Err(DatabaseError::NotAllowed);
            } else {
                if let Err(_) = self
                    .auth
                    .audit(
                        ua.id.to_owned(),
                        format!("Deleted a post: {}", existing.slug),
                    )
                    .await
                {
                    return Err(DatabaseError::Other);
                }

                skip_password_check = true
            }
        }

        if !skip_password_check {
            if utility::hash(password) != existing.password {
                return Err(DatabaseError::PasswordIncorrect);
            }
        }

        // delete post view count
        self.base
            .cachedb
            .remove(format!("{}:{}", self.config.table_views.prefix, slug))
            .await;

        // delete post
        let query: String = if (self.base.db.r#type == "sqlite") | (self.base.db.r#type == "mysql")
        {
            "DELETE FROM \":t\" WHERE \":slug\" = ?"
        } else {
            "DELETE FROM \":t\" WHERE \":slug\" = $1"
        }
        .to_string()
        .replace(":t", &self.config.table_posts.table_name)
        .replace(":slug", &self.config.table_posts.slug);

        let c = &self.base.db.client;
        match sqlquery(&query).bind::<&String>(&slug).execute(c).await {
            Ok(_) => {
                // remove from cache
                self.base
                    .cachedb
                    .remove(format!("{}:{}", self.config.table_posts.prefix, slug))
                    .await;

                if self.config.view_mode == ViewMode::AuthenticatedOnce {
                    // delete all view logs
                    let query: String =
                        if (self.base.db.r#type == "sqlite") | (self.base.db.r#type == "mysql") {
                            "DELETE FROM \":t\" WHERE \"slug\" = ?"
                        } else {
                            "DELETE FROM \":t\" WHERE \"slug\" = $1"
                        }
                        .replace(":t", &self.config.table_views.table_name);

                    if let Err(_) = sqlquery(&query).bind::<&String>(&slug).execute(c).await {
                        return Err(DatabaseError::Other);
                    };
                }

                // return
                return Ok(());
            }
            Err(_) => return Err(DatabaseError::Other),
        };
    }

    /// Edit an existing post
    ///
    /// # Arguments
    /// * `slug` - the post to edit
    /// * `ip` - the IP address of the user editing this post
    /// * `password` - the post's edit password
    /// * `new_content` - the new content of the post
    /// * `new_slug` - the new slug of the post
    /// * `new_password` - the new password of the post
    pub async fn edit_post(
        &self,
        mut slug: String,
        ip: String,
        password: String,
        new_content: String,
        mut new_slug: String,
        mut new_password: String,
        user: Option<Box<Profile>>,
    ) -> Result<()> {
        slug = idna::punycode::encode_str(&slug).unwrap().to_lowercase();

        if slug.ends_with("-") {
            slug.pop();
        }

        // get post
        let mut existing = match self.get_post(slug.clone()).await {
            Ok(p) => p,
            Err(err) => return Err(err),
        };

        // check password
        let mut skip_password_check = false;

        if let Some(ref ua) = user {
            // check permission
            let group = match self.auth.get_group_by_id(ua.group).await {
                Ok(g) => g,
                Err(_) => return Err(DatabaseError::Other),
            };

            if !group
                .permissions
                .contains(&authbeam::model::Permission::Manager)
            {
                return Err(DatabaseError::NotAllowed);
            } else {
                if let Err(_) = self
                    .auth
                    .audit(
                        ua.id.to_owned(),
                        format!("Edited a post: {}", existing.slug),
                    )
                    .await
                {
                    return Err(DatabaseError::Other);
                }

                skip_password_check = true
            }
        }

        if !skip_password_check {
            if utility::hash(password) != existing.password {
                return Err(DatabaseError::PasswordIncorrect);
            }
        }

        // hash new password
        if !new_password.is_empty() {
            new_password = utility::hash(new_password);
        } else {
            new_password = existing.password;
        }

        // update new_slug
        if new_slug.is_empty() {
            new_slug = existing.slug;
        }

        new_slug = idna::punycode::encode_str(&new_slug).unwrap();

        if new_slug.ends_with("-") {
            new_slug.pop();
        }

        // push ip
        existing.ips.push((utility::unix_epoch_timestamp(), ip));

        // we only want the last 10 ips... if there are now 11, remove the first
        if existing.ips.len() >= 11 {
            existing.ips.remove(0);
        }

        // edit post
        let query: String = if (self.base.db.r#type == "sqlite") | (self.base.db.r#type == "mysql") {
            "UPDATE \":t\" SET \":content\" = ?, \":password\" = ?, \":slug\" = ?, \":date_edited\" = ?, \":ips\" = ? WHERE \":slug\" = ?"
        } else {
            "UPDATE \":t\" SET (\":content\" = $1, \":password\" = $2, \":slug\" = $3, \":date_edited\" = $4, \":ips\" = $5) WHERE \":slug\" = $6"
        }
        .to_string()
        .replace(":t", &self.config.table_posts.table_name)
        .replace(":slug", &self.config.table_posts.slug)
        .replace(":content", &self.config.table_posts.content)
        .replace(":password", &self.config.table_posts.password)
        .replace(":date_edited", &self.config.table_posts.date_edited)
        .replace(":ips", &self.config.table_posts.ips);

        let c = &self.base.db.client;
        match sqlquery(&query)
            .bind::<&String>(&new_content)
            .bind::<&String>(&new_password)
            .bind::<&String>(&new_slug)
            .bind::<&String>(&utility::unix_epoch_timestamp().to_string())
            .bind::<&String>(match serde_json::to_string(&existing.ips) {
                Ok(ref m) => m,
                Err(_) => return Err(DatabaseError::ValueError),
            })
            .bind::<&String>(&slug)
            .execute(c)
            .await
        {
            Ok(_) => {
                // remove from cache
                self.base
                    .cachedb
                    .remove(format!("{}:{}", self.config.table_posts.prefix, slug))
                    .await;

                // return
                return Ok(());
            }
            Err(_) => return Err(DatabaseError::Other),
        };
    }

    /// Edit an existing post's context by `slug`
    ///
    /// # Arguments
    /// * `slug` - the post to edit
    /// * `password` - the post's edit password
    /// * `context` - the new context of the post
    pub async fn edit_post_context(
        &self,
        mut slug: String,
        password: String,
        context: PostContext,
        user: Option<Box<Profile>>,
    ) -> Result<()> {
        slug = idna::punycode::encode_str(&slug).unwrap().to_lowercase();

        if slug.ends_with("-") {
            slug.pop();
        }

        // get post
        let existing = match self.get_post(slug.clone()).await {
            Ok(p) => p,
            Err(err) => return Err(err),
        };

        // check password
        let mut skip_password_check = false;

        if let Some(ref ua) = user {
            // check permission
            let group = match self.auth.get_group_by_id(ua.group).await {
                Ok(g) => g,
                Err(_) => return Err(DatabaseError::Other),
            };

            if !group
                .permissions
                .contains(&authbeam::model::Permission::Manager)
            {
                return Err(DatabaseError::NotAllowed);
            } else {
                if let Err(_) = self
                    .auth
                    .audit(
                        ua.id.to_owned(),
                        format!("Edited a post's context: {}", existing.slug),
                    )
                    .await
                {
                    return Err(DatabaseError::Other);
                }

                skip_password_check = true
            }
        }

        if !skip_password_check {
            if utility::hash(password) != existing.password {
                return Err(DatabaseError::PasswordIncorrect);
            }
        }

        // edit post
        let query: String = if (self.base.db.r#type == "sqlite") | (self.base.db.r#type == "mysql")
        {
            "UPDATE \":t\" SET \":metadata\" = ? WHERE \":slug\" = ?"
        } else {
            "UPDATE \":t\" SET (\":metadata\" = $1) WHERE \":slug\" = $2"
        }
        .to_string()
        .replace(":t", &self.config.table_posts.table_name)
        .replace(":slug", &self.config.table_posts.slug)
        .replace(":metadata", &self.config.table_posts.context);

        let c = &self.base.db.client;
        match sqlquery(&query)
            .bind::<&String>(match serde_json::to_string(&context) {
                Ok(ref m) => m,
                Err(_) => return Err(DatabaseError::ValueError),
            })
            .bind::<&String>(&slug)
            .execute(c)
            .await
        {
            Ok(_) => {
                // remove from cache
                self.base
                    .cachedb
                    .remove(format!("{}:{}", self.config.table_posts.prefix, slug))
                    .await;

                // return
                return Ok(());
            }
            Err(_) => return Err(DatabaseError::Other),
        };
    }

    // views

    /// Get an existing post's view count
    ///
    /// # Arguments
    /// * `slug` - the post to count the view for
    pub async fn get_views_by_slug(&self, mut slug: String) -> i32 {
        slug = idna::punycode::encode_str(&slug).unwrap().to_lowercase();

        if slug.ends_with("-") {
            slug.pop();
        }

        // get views
        match self
            .base
            .cachedb
            .get(format!("{}:{}", self.config.table_views.prefix, slug))
            .await
        {
            Some(c) => c.parse::<i32>().unwrap(),
            None => {
                // try to count from "views"
                if self.config.view_mode == ViewMode::AuthenticatedOnce {
                    let query: String =
                        if (self.base.db.r#type == "sqlite") | (self.base.db.r#type == "mysql") {
                            "SELECT * FROM \":t\" WHERE \"slug\" = ?"
                        } else {
                            "SELECT * FROM \":t\" WHERE \"slug\" = $1"
                        }
                        .to_string()
                        .replace(":t", &self.config.table_views.table_name);

                    let c = &self.base.db.client;
                    match sqlquery(&query).bind::<&String>(&slug).fetch_all(c).await {
                        Ok(views) => {
                            let views = views.len();

                            // store in cache
                            self.base
                                .cachedb
                                .set(
                                    format!("{}:{}", self.config.table_views.prefix, slug),
                                    views.to_string(),
                                )
                                .await;

                            // return
                            return views as i32;
                        }
                        Err(_) => return 0,
                    };
                }

                // return 0 by default
                0
            }
        }
    }

    /// Update an existing post's view count
    ///
    /// # Arguments
    /// * `slug` - the slug to count the view for
    pub async fn incr_views_by_slug(&self, mut slug: String) -> Result<()> {
        slug = idna::punycode::encode_str(&slug).unwrap().to_lowercase();

        if slug.ends_with("-") {
            slug.pop();
        }

        // add view
        // views never reach the database, they're only stored in memory
        match self
            .base
            .cachedb
            .incr(format!("{}:{}", self.config.table_views.prefix, slug))
            .await
        {
            // swapped for some reason??
            false => Ok(()),
            true => Err(DatabaseError::Other),
        }
    }

    /// Check if a user has viewed a post
    ///
    /// # Arguments
    /// * `slug` - the post slug
    /// * `id` - the id of the user
    pub async fn user_has_viewed_post(&self, slug: String, id: String) -> bool {
        if self.config.view_mode == ViewMode::AuthenticatedOnce {
            let query: String =
                if (self.base.db.r#type == "sqlite") | (self.base.db.r#type == "mysql") {
                    "SELECT * FROM \":t\" WHERE \"slug\" = ? AND \"id\" = ?"
                } else {
                    "SELECT * FROM \":t\" WHERE \"slug\" = $1 AND \"id\" = ?"
                }
                .to_string()
                .replace(":t", &self.config.table_views.table_name);

            let c = &self.base.db.client;
            match sqlquery(&query)
                .bind::<&String>(&slug)
                .bind::<&String>(&id)
                .fetch_one(c)
                .await
            {
                Ok(_) => return true,
                Err(_) => return false,
            };
        }

        false
    }
}
