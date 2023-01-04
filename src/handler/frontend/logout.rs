use actix_toolbox::tb_middleware::Session;
use actix_web::web::Json;
use serde::Serialize;

use crate::handler::frontend;

#[derive(Serialize)]
pub(crate) struct LogoutResponse {
    success: bool,
}

pub(crate) async fn logout(session: Session) -> frontend::Result<Json<LogoutResponse>> {
    session.purge();

    Ok(Json(LogoutResponse { success: true }))
}
