use actix_toolbox::logging::LoggingConfig;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct Server {
    pub(crate) listen_address: String,
    pub(crate) listen_port: u16,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct DBConfig {
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct Config {
    pub(crate) server: Server,
    pub(crate) database: DBConfig,
    pub(crate) logging: LoggingConfig,
}
