use std::fmt::{Debug, Display, Formatter};

use actix_toolbox::tb_middleware::actix_session::{SessionGetError, SessionInsertError};
use actix_web::body::BoxBody;
use actix_web::HttpResponse;
use log::error;
use serde::Serialize;
use serde_repr::Serialize_repr;

pub(crate) use login::login;
pub(crate) use logout::logout;

pub(crate) mod login;
pub(crate) mod logout;

#[derive(Serialize_repr)]
#[repr(u16)]
pub(crate) enum ErrorStatusCode {
    LoginFailed = 100,
    Unauthenticated = 101,
    DatabaseError = 500,
    InternalServerError = 501,
    SessionError = 502,
}

#[derive(Serialize)]
pub(crate) struct ErrorResponse {
    success: bool,
    status_code: ErrorStatusCode,
    message: String,
}

impl ErrorResponse {
    fn new(status_code: ErrorStatusCode, message: String) -> Self {
        Self {
            success: false,
            status_code,
            message,
        }
    }
}

pub(crate) type Result<T> = std::result::Result<T, Errors>;

#[derive(Debug)]
pub(crate) enum SessionErrors {
    InsertError(SessionInsertError),
    GetError(SessionGetError),
}

#[derive(Debug)]
pub(crate) enum Errors {
    LoginFailed,
    Unauthenticated,
    DatabaseError(rorm::Error),
    HashError(argon2::password_hash::Error),
    SessionError(SessionErrors),
}

impl Display for Errors {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Errors::DatabaseError(_) => write!(f, "Database error occurred"),
            Errors::HashError(_) => write!(f, "Internal server error occurred"),
            Errors::SessionError(_) => write!(f, "Error while accessing session"),
            Errors::LoginFailed => write!(f, "Invalid username / password"),
            Errors::Unauthenticated => write!(f, "Unauthenticated"),
        }
    }
}

impl actix_web::ResponseError for Errors {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        match self {
            Errors::DatabaseError(err) => {
                error!("Database error: {err}");

                HttpResponse::Ok().json(ErrorResponse::new(
                    ErrorStatusCode::DatabaseError,
                    self.to_string(),
                ))
            }
            Errors::HashError(err) => {
                error!("Hash error: {err}");

                HttpResponse::Ok().json(ErrorResponse::new(
                    ErrorStatusCode::InternalServerError,
                    self.to_string(),
                ))
            }
            Errors::SessionError(err) => {
                match err {
                    SessionErrors::InsertError(err) => error!("Session insert error: {err}"),
                    SessionErrors::GetError(err) => error!("Session insert error: {err}"),
                };

                HttpResponse::Ok().json(ErrorResponse::new(
                    ErrorStatusCode::SessionError,
                    self.to_string(),
                ))
            }
            Errors::LoginFailed => HttpResponse::Ok().json(ErrorResponse::new(
                ErrorStatusCode::LoginFailed,
                self.to_string(),
            )),
            Errors::Unauthenticated => HttpResponse::Ok().json(ErrorResponse::new(
                ErrorStatusCode::Unauthenticated,
                self.to_string(),
            )),
        }
    }
}

impl From<argon2::password_hash::Error> for Errors {
    fn from(value: argon2::password_hash::Error) -> Self {
        Errors::HashError(value)
    }
}

impl From<rorm::Error> for Errors {
    fn from(value: rorm::Error) -> Self {
        Errors::DatabaseError(value)
    }
}

impl From<SessionInsertError> for Errors {
    fn from(value: SessionInsertError) -> Self {
        Errors::SessionError(SessionErrors::InsertError(value))
    }
}

impl From<SessionGetError> for Errors {
    fn from(value: SessionGetError) -> Self {
        Errors::SessionError(SessionErrors::GetError(value))
    }
}
