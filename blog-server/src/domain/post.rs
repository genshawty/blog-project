use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::errors::UserError;

#[derive(Clone)]
pub struct Post {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub author_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Post {
    pub fn new(title: String, content: String, author_id: Uuid) -> Result<Self, UserError> {
        Ok(Self {
            id: Uuid::new_v4(),
            title,
            content,
            author_id,
            created_at: Utc::now(),
            updated_at: None,
        })
    }

    pub fn update_title(&mut self, new_title: String) {
        self.title = new_title;
        self.updated_at = Some(Utc::now());
    }
    pub fn update_content(&mut self, new_content: String) {
        self.content = new_content;
        self.updated_at = Some(Utc::now());
    }

    pub fn update(&mut self, new_title: String, new_content: String) {
        self.update_title(new_title);
        self.update_content(new_content);
    }
}
