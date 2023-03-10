use actix_toolbox::tb_middleware::{
    setup_logging_mw, DBSessionStore, LoggingMiddlewareConfig, PersistentSession, SessionMiddleware,
};
use actix_web::cookie::time::Duration;
use actix_web::cookie::Key;
use actix_web::middleware::Compress;
use actix_web::web::{get, post, scope, Data, JsonConfig, PayloadConfig};
use actix_web::{App, HttpServer};
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use rorm::Database;

use crate::handler::{frontend, world};
use crate::helper::AuthenticationRequired;
use crate::models::config::Config;
use crate::world::OSMTags;

pub(crate) async fn start_server(db: Database, config: Config) -> Result<(), String> {
    let key = match BASE64_STANDARD.decode(config.server.secret_key) {
        Ok(data) => match Key::try_from(data.as_slice()) {
            Ok(v) => v,
            Err(err) => {
                return Err(format!(
                    "Invalid parameter SecretKey: {err}.\
                    Consider using the subcommand gen-key and update your configuration file"
                ));
            }
        },
        Err(err) => {
            return Err(format!("{err}"));
        }
    };

    let tags_lookup = Data::new(OSMTags::new());

    HttpServer::new(move || {
        App::new()
            .wrap(
                SessionMiddleware::builder(DBSessionStore::new(db.clone()), key.clone())
                    .cookie_secure(false)
                    .session_lifecycle(PersistentSession::session_ttl(
                        PersistentSession::default(),
                        Duration::hours(1),
                    ))
                    .build(),
            )
            .wrap(Compress::default())
            .wrap(setup_logging_mw(LoggingMiddlewareConfig::default()))
            .app_data(tags_lookup.clone())
            .app_data(JsonConfig::default())
            .app_data(PayloadConfig::default())
            .app_data(Data::new(db.clone()))
            .route("/api/world/v1/getOsmTags", get().to(world::get_osm_tags))
            .route("/api/frontend/v1/login", post().to(frontend::login))
            .service(
                scope("/api/frontend/v1")
                    .wrap(AuthenticationRequired)
                    .route("logout", get().to(frontend::logout)),
            )
    })
    .bind((
        config.server.listen_address.as_str(),
        config.server.listen_port,
    ))
    .map_err(|e| e.to_string())?
    .run()
    .await
    .map_err(|e| e.to_string())
}
