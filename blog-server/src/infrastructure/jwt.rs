use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone)]
pub struct JwtKeys {
    secret: String,
}

impl JwtKeys {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    pub fn generate_token(
        &self,
        user_id: Uuid,
        username: String,
    ) -> Result<String, jsonwebtoken::errors::Error> {
        let claims = Claims {
            user_id: user_id.to_string(),
            username: username,
            exp: chrono::Utc::now()
                .checked_add_signed(chrono::Duration::hours(1))
                .unwrap()
                .timestamp() as usize,
            iat: chrono::Utc::now().timestamp() as usize,
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )?;
        Ok(data.claims)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: String,
    pub username: String,
    pub exp: usize,
    pub iat: usize,
}
