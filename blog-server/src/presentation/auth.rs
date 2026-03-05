use actix_web::dev::Payload;
use actix_web::{Error, FromRequest, HttpMessage, HttpRequest, error::ErrorUnauthorized};
use std::future::{Ready, ready};
use uuid::Uuid;

use crate::data::user_repository::UserRepository;
use crate::infrastructure::jwt::JwtKeys;

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: Uuid,
    #[allow(dead_code)]
    pub email: String,
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
    user_repo: &UserRepository,
) -> Result<AuthenticatedUser, Error> {
    let claims = keys
        .verify_token(token)
        .map_err(|_| ErrorUnauthorized("invalid token"))?;
    let user_id =
        Uuid::parse_str(&claims.user_id).map_err(|_| ErrorUnauthorized("invalid token"))?;
    let user = user_repo
        .find_by_id(user_id)
        .await
        .map_err(|_| ErrorUnauthorized("user lookup failed"))?
        .ok_or_else(|| ErrorUnauthorized("user not found"))?;

    Ok(AuthenticatedUser {
        id: user.id,
        email: user.email,
    })
}
