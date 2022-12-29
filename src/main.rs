use std::fs::read_to_string;
use std::ops::Sub;

use actix_toolbox::logging::setup_logging;
use actix_web::cookie::Key;
use chrono::Utc;
use clap::{Parser, Subcommand};
use futures::{stream, StreamExt};
use log::{error, info};
use rorm::{insert, Database, DatabaseConfiguration, DatabaseDriver};
use rustymon_world::features::prototyping;

use crate::models::config::Config;
use crate::models::db::TileInsert;
use crate::server::start_server;

mod models;
mod server;

const SPAWNS: &str = include_str!("../data/spawns.json");

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
        #[clap(long)]
        center_y: f64,

        /// Latitude of center
        #[clap(long)]
        center_x: f64,

        /// Number of columns
        #[clap(short, long, value_parser, default_value_t = 32)]
        cols: usize,

        /// Number of rows
        #[clap(short, long, value_parser, default_value_t = 32)]
        rows: usize,

        /// Zoom level to produce tiles for
        #[clap(short, long, default_value_t = 14)]
        zoom: u8,
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
        min_connections: 2,
        disable_logging: None,
        slow_statement_log_level: None,
        statement_log_level: None,
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
            println!("{}", base64::encode(key.master()));

            Ok(())
        }
        Command::ParseOSM {
            file,
            cols,
            rows,
            center_x,
            center_y,
            zoom,
            config_path,
        } => {
            let config = get_config(&config_path)?;
            let db = init_db(&config).await?;

            let start = Utc::now();

            let tiles = rustymon_world::parse(rustymon_world::Config {
                zoom,
                center_x,
                center_y,
                rows,
                cols,
                file,
                visual: prototyping::Parser::from_file(SPAWNS).unwrap(),
            })?;

            let after_osm = Utc::now();

            println!("After OSM: {}", after_osm.sub(start));

            stream::iter(tiles)
                .chunks(25)
                .for_each_concurrent(None, |chunk| async {
                    let db_tiles: Vec<_> = chunk
                        .into_iter()
                        .map(|t| TileInsert {
                            min_x: t.min.x,
                            min_y: t.min.y,
                            max_x: t.max.x,
                            max_y: t.max.y,
                        })
                        .collect();

                    insert!(&db, TileInsert)
                        .bulk(&db_tiles)
                        .await
                        .expect("Failed to insert chunk of tiles");
                })
                .await;

            println!("DB Insert: {}", Utc::now().sub(after_osm));

            Ok(())
        }
    }
}
