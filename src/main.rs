#![cfg_attr(
    feature = "rorm-main",
    allow(unused_variables, unused_imports, dead_code)
)]

use std::fs::read_to_string;

use actix_toolbox::logging::setup_logging;
use actix_web::cookie::Key;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use clap::{Parser, Subcommand};
use log::{error, info, LevelFilter};
use rorm::{Database, DatabaseConfiguration, DatabaseDriver};

use crate::models::config::Config;
use crate::server::start_server;

mod handler;
mod helper;
mod models;
mod parse_osm;
mod server;
mod world;

const LOGO: &str = r#" ______
|  ___ \            _
| |___) |_   _  ___| |_ _   _ ____   ___  ____
|  __  /| | | |/___)  _) | | |    \ / _ \|  _ \
| |  \ \\ |_| |___ | |_| |_| | | | | |_| | | | |
|_|   \_|\____(___/ \___)__  |_|_|_|\___/|_| |_|
                       (____/   & a bunch of other languages"#;

#[derive(Subcommand)]
enum Command {
    Start {
        #[clap(default_value_t = String::from("/etc/rustymon-server/config.toml"))]
        #[clap(long = "config-path")]
        #[clap(help = "Specify an alternative path to the configuration file.")]
        config_path: String,
    },
    GenKey,
    ParseOSM {
        #[clap(default_value_t = String::from("/etc/rustymon-server/config.toml"))]
        #[clap(long = "config-path")]
        #[clap(help = "Specify an alternative path to the configuration file.")]
        config_path: String,

        /// PBF file to parse
        #[clap(help = "PBF file to parse")]
        #[clap(long)]
        file: String,

        /// Longitude of center
        #[clap(long, help = "Longitude of center point to use")]
        center_y: f64,

        /// Latitude of center
        #[clap(long, help = "Latitude of center point to use")]
        center_x: f64,

        /// Number of columns
        #[clap(long, value_parser, default_value_t = 32)]
        #[clap(help = "Number of columns to generate")]
        cols: usize,

        /// Number of rows
        #[clap(long, value_parser, default_value_t = 32)]
        #[clap(help = "Number of rows to generate")]
        rows: usize,
    },
}

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

fn get_config(config_path: &str) -> Result<Config, String> {
    let config_string = read_to_string(config_path)
        .map_err(|e| format!("Could not read from configuration file: {e}"))?;
    toml::from_str(&config_string).map_err(|e| format!("Could not parse configuration file: {e}"))
}

async fn init_db(config: &Config) -> Result<Database, String> {
    Database::connect(DatabaseConfiguration {
        driver: DatabaseDriver::Postgres {
            name: config.database.name.clone(),
            host: config.database.host.clone(),
            port: config.database.port,
            user: config.database.username.clone(),
            password: config.database.password.clone(),
        },
        max_connections: 20,
        min_connections: 4,
        disable_logging: None,
        slow_statement_log_level: None,
        statement_log_level: Some(LevelFilter::Off),
    })
    .await
    .map_err(|e| format!("{e}"))
}

#[rorm::rorm_main]
#[tokio::main]
async fn main() -> Result<(), String> {
    let cli = Cli::parse();

    match cli.command {
        Command::Start { config_path } => {
            println!("\n{}\n", LOGO);

            let config = get_config(&config_path)?;
            setup_logging(&config.logging)?;

            info!("Logging is ready.");

            let db = init_db(&config).await.map_err(|e| {
                error!("Error while initializing database: {e}");
                e
            })?;
            info!("Initialized database connection.");

            start_server(db, config).await.map_err(|e| {
                error!("Error while starting server: {e}");
                e
            })
        }
        Command::GenKey => {
            let key = Key::generate();
            println!("Generated key:");
            println!("{}", BASE64_STANDARD.encode(key.master()));

            Ok(())
        }
        Command::ParseOSM {
            file,
            cols,
            rows,
            center_x,
            center_y,
            config_path,
        } => {
            let config = get_config(&config_path)?;
            let db = init_db(&config).await?;

            parse_osm::parse_osm(db, file, cols, rows, center_x, center_y).await
        }
    }
}
