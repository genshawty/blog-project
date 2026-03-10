use actix_web::dev::Payload;
use actix_web::{Error, FromRequest, HttpMessage, HttpRequest, error::ErrorUnauthorized};
use std::future::{Ready, ready};
use uuid::Uuid;

use crate::application::auth_service::AuthService;
use crate::infrastructure::jwt::JwtKeys;

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: Uuid,
    #[allow(dead_code)]
    pub username: String,
}

impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        if let Some(user) = req.extensions().get::<AuthenticatedUser>() {
            return ready(Ok(user.clone()));
        }
        ready(Err(ErrorUnauthorized("missing authenticated user")))
    }
}

pub async fn extract_user_from_token(
    token: &str,
    keys: &JwtKeys,
    auth_service: &AuthService,
) -> Result<AuthenticatedUser, Error> {
    let claims = keys
        .verify_token(token)
        .map_err(|_| ErrorUnauthorized("invalid token"))?;
    let user_id =
        Uuid::parse_str(&claims.user_id).map_err(|_| ErrorUnauthorized("invalid token"))?;
    let username = claims.username.clone();

    Ok(AuthenticatedUser {
        id: user_id,
        username: username,
    })
}
