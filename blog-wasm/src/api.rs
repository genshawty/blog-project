use reqwest::Client;
use serde::{Deserialize, Serialize};

const BASE_URL: &str = "http://localhost:8080";

#[derive(Serialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct CreatePostRequest {
    pub title: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct UpdatePostRequest {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthUserInfo {
    pub id: String,
    pub username: String,
    pub email: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: AuthUserInfo,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostResponse {
    pub id: String,
    pub author_id: String,
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostListResponse {
    pub posts: Vec<PostResponse>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Clone)]
pub struct ApiClient {
    client: Client,
    base_url: String,
}

impl ApiClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: BASE_URL.to_string(),
        }
    }

    pub fn with_url(mut self, url: &str) -> Self {
        self.base_url = url.to_owned();
        self
    }

    pub async fn register(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<AuthResponse, String> {
        let resp = self
            .client
            .post(format!("{}/api/auth/register", self.base_url))
            .json(&RegisterRequest {
                username: username.to_string(),
                email: email.to_string(),
                password: password.to_string(),
            })
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if resp.status() == 201 {
            resp.json::<AuthResponse>().await.map_err(|e| e.to_string())
        } else if resp.status() == 409 {
            Err("User already exists".to_string())
        } else {
            Err(format!("Register failed: {}", resp.status()))
        }
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<AuthResponse, String> {
        let resp = self
            .client
            .post(format!("{}/api/auth/login", self.base_url))
            .json(&LoginRequest {
                username: username.to_string(),
                password: password.to_string(),
            })
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if resp.status() == 200 {
            resp.json::<AuthResponse>().await.map_err(|e| e.to_string())
        } else if resp.status() == 401 {
            Err("Wrong password".to_string())
        } else {
            Err(format!("Login failed: {}", resp.status()))
        }
    }

    pub async fn list_posts(&self, limit: i64, offset: i64) -> Result<PostListResponse, String> {
        let resp = self
            .client
            .get(format!(
                "{}/api/posts?limit={}&offset={}",
                self.base_url, limit, offset
            ))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if resp.status() == 200 {
            resp.json::<PostListResponse>()
                .await
                .map_err(|e| e.to_string())
        } else {
            Err(format!("List posts failed: {}", resp.status()))
        }
    }

    pub async fn create_post(
        &self,
        token: &str,
        title: &str,
        content: &str,
    ) -> Result<PostResponse, String> {
        let resp = self
            .client
            .post(format!("{}/api/posts", self.base_url))
            .bearer_auth(token)
            .json(&CreatePostRequest {
                title: title.to_string(),
                content: content.to_string(),
            })
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if resp.status() == 201 {
            resp.json::<PostResponse>().await.map_err(|e| e.to_string())
        } else if resp.status() == 401 {
            Err("Unauthorized".to_string())
        } else {
            Err(format!("Create post failed: {}", resp.status()))
        }
    }

    pub async fn update_post(
        &self,
        token: &str,
        post_id: &str,
        title: &str,
        content: &str,
    ) -> Result<PostResponse, String> {
        let resp = self
            .client
            .put(format!("{}/api/posts/{}", self.base_url, post_id))
            .bearer_auth(token)
            .json(&UpdatePostRequest {
                title: title.to_string(),
                content: content.to_string(),
            })
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if resp.status() == 200 {
            resp.json::<PostResponse>().await.map_err(|e| e.to_string())
        } else if resp.status() == 401 {
            Err("Unauthorized".to_string())
        } else if resp.status() == 403 {
            Err("Forbidden: not the author".to_string())
        } else if resp.status() == 404 {
            Err("Post not found".to_string())
        } else {
            Err(format!("Update post failed: {}", resp.status()))
        }
    }

    pub async fn delete_post(&self, token: &str, post_id: &str) -> Result<(), String> {
        let resp = self
            .client
            .delete(format!("{}/api/posts/{}", self.base_url, post_id))
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if resp.status() == 204 {
            Ok(())
        } else if resp.status() == 401 {
            Err("Unauthorized".to_string())
        } else if resp.status() == 403 {
            Err("Forbidden: not the author".to_string())
        } else if resp.status() == 404 {
            Err("Post not found".to_string())
        } else {
            Err(format!("Delete post failed: {}", resp.status()))
        }
    }
}
