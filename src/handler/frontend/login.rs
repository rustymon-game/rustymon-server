use actix_toolbox::tb_middleware::Session;
use actix_web::web::{Data, Json};
use argon2::password_hash::Error;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use rorm::{query, Database, Model};
use serde::{Deserialize, Serialize};

use crate::handler::frontend;
use crate::handler::frontend::Errors;
use crate::models::db::User;

#[derive(Deserialize)]
pub(crate) struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
pub(crate) struct LoginResponse {
    success: bool,
}

const PW: &[u8] = "super-secure-password".as_bytes();
const HASH: &str = "$argon2id$v=19$m=16,t=2,p=1$RG9RY0ZxeWs0N3RjTDZ5cQ$FDFDJzWVOJiBvE/Sg9HjMw";

pub(crate) async fn login(
    db: Data<Database>,
    session: Session,
    req: Json<LoginRequest>,
) -> frontend::Result<Json<LoginResponse>> {
    let hasher = Argon2::default();

    let Some(user) = query!(&db, User)
        .condition(User::F.username.equals(&req.username))
        .optional()
        .await? else {
        // Run hash check to protect against enumeration via request time
        let h = PasswordHash::new(HASH)?;
        hasher.verify_password(PW, &h)?;
        return Err(Errors::LoginFailed);
    };

    if let Err(err) = hasher.verify_password(
        req.password.as_bytes(),
        &PasswordHash::new(&user.password_hash)?,
    ) {
        return match err {
            Error::Password => Err(Errors::LoginFailed),
            _ => Err(err)?,
        };
    }

    session.insert("logged_in", true)?;
    session.insert("user", &user.username)?;

    Ok(Json(LoginResponse { success: true }))
}
