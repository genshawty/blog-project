use thiserror::Error;

#[derive(Debug, Error)]
pub enum UserError {
    #[error("wrong email format")]
    WrongEmailFormat,

    #[error("internal user domain error")]
    InternalError,
}

#[derive(Debug, Error)]
pub enum BlogError {
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("user already exists")]
    UserAlreadyExists,
    #[error("user not found: {0}")]
    UserNotFound(uuid::Uuid),
    #[error("post not found: {0}")]
    PostNotFound(uuid::Uuid),
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("internal error: {0}")]
    Internal(String),
}

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("user already exists")]
    UserAlreadyExists,
    #[error("forbidden")]
    Forbidden,
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("insufficient funds on account {0}")]
    InsufficientFunds(u32),
    #[error("account not found: {0}")]
    AccountNotFound(u32),
    #[error("user not found: {0}")]
    UserNotFound(uuid::Uuid),
    #[error("post not found: {0}")]
    PostNotFound(uuid::Uuid),
    #[error("internal error: {0}")]
    Internal(String),
}
