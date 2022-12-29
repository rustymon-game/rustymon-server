use std::fs::read_to_string;

use actix_toolbox::logging::setup_logging;
use clap::Parser;
use log::{error, info};
use rorm::{Database, DatabaseConfiguration, DatabaseDriver};

use crate::models::config::Config;
use crate::server::start_server;

mod models;
mod server;

#[derive(Parser)]
struct Cli {
    #[clap(default_value_t = String::from("/etc/rustymon-server/config.toml"))]
    #[clap(long = "config-path")]
    #[clap(help = "Specify an alternative path to the configuration file.")]
    config_path: String,
}

#[rorm::rorm_main]
#[tokio::main]
async fn main() -> Result<(), String> {
    let cli = Cli::parse();

    let config_string = read_to_string(cli.config_path)
        .map_err(|e| format!("Could not read from configuration file: {e}"))?;
    let config: Config = toml::from_str(&config_string)
        .map_err(|e| format!("Could not parse configuration file: {e}"))?;

    setup_logging(&config.logging)?;

    info!("Logging is ready.");

    let db = match Database::connect(DatabaseConfiguration {
        driver: DatabaseDriver::Postgres {
            name: config.database.name.clone(),
            host: config.database.host.clone(),
            port: config.database.port,
            user: config.database.username.clone(),
            password: config.database.password.clone(),
        },
        max_connections: 20,
        min_connections: 2,
        disable_logging: None,
        slow_statement_log_level: None,
        statement_log_level: None,
    })
    .await
    {
        Ok(v) => v,
        Err(err) => {
            error!("Couldn't start server: {err}");
            return Err(err.to_string());
        }
    };

    info!("Initialized database connection.");

    start_server(db, config).await.map_err(|e| {
        error!("Error while starting server: {e}");
        format!("{e}")
    })?;

    Ok(())
}
