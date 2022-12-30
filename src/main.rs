use std::collections::HashMap;
use std::fs::read_to_string;
use std::ops::Sub;
use std::slice;

use actix_toolbox::logging::setup_logging;
use actix_web::cookie::Key;
use chrono::Utc;
use clap::{Parser, Subcommand};
use futures::{stream, StreamExt};
use log::{error, info};
use rorm::internal::field::foreign_model::ForeignModelByField;
use rorm::{insert, Database, DatabaseConfiguration, DatabaseDriver};
use rustymon_world::features::prototyping;

use crate::models::config::Config;
use crate::models::db::{AreaInsert, NodeInsert, TileInsert, WayInsert};
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
                    let chunk = chunk;

                    let mut tiles = HashMap::new();
                    let mut ways = HashMap::new();
                    let mut nodes = HashMap::new();
                    let mut areas = HashMap::new();

                    chunk.iter().enumerate().for_each(|(idx, t)| {
                        tiles.insert(
                            idx,
                            TileInsert {
                                min_x: t.min.x,
                                min_y: t.min.y,
                                max_x: t.max.x,
                                max_y: t.max.y,
                            },
                        );

                        t.iter_areas().for_each(|a| {
                            areas.insert(
                                idx,
                                AreaInsert {
                                    tile: ForeignModelByField::Key(0),
                                    points: unsafe {
                                        slice::from_raw_parts(
                                            a.points.as_ptr() as *const u8,
                                            a.points.len(),
                                        )
                                    }
                                    .to_vec(),
                                    features: unsafe {
                                        slice::from_raw_parts(
                                            a.feature.as_ptr() as *const u8,
                                            a.feature.len(),
                                        )
                                    }
                                    .to_vec(),
                                },
                            );
                        });

                        t.iter_nodes().for_each(|n| {
                            nodes.insert(
                                idx,
                                NodeInsert {
                                    tile: ForeignModelByField::Key(0),
                                    x: n.points.x,
                                    y: n.points.y,
                                    features: unsafe {
                                        slice::from_raw_parts(
                                            n.feature.as_ptr() as *const u8,
                                            n.feature.len(),
                                        )
                                    }
                                    .to_vec(),
                                },
                            );
                        });

                        t.iter_ways().for_each(|w| {
                            ways.insert(
                                idx,
                                WayInsert {
                                    tile: ForeignModelByField::Key(0),
                                    points: unsafe {
                                        slice::from_raw_parts(
                                            w.points.as_ptr() as *const u8,
                                            w.points.len(),
                                        )
                                    }
                                    .to_vec(),
                                    features: unsafe {
                                        slice::from_raw_parts(
                                            w.feature.as_ptr() as *const u8,
                                            w.feature.len(),
                                        )
                                    }
                                    .to_vec(),
                                },
                            );
                        });
                    });

                    let mut tx = db
                        .start_transaction()
                        .await
                        .expect("Could not start transaction");

                    let tile_ids = insert!(&db, TileInsert)
                        .transaction(&mut tx)
                        .bulk(tiles.values())
                        .await
                        .expect("Error while creating tiles");

                    tile_ids.into_iter().enumerate().for_each(|(idx, i)| {
                        if let Some(way) = ways.get_mut(&idx) {
                            way.tile = ForeignModelByField::Key(i);
                        }
                        if let Some(node) = nodes.get_mut(&idx) {
                            node.tile = ForeignModelByField::Key(i);
                        }
                        if let Some(area) = areas.get_mut(&idx) {
                            area.tile = ForeignModelByField::Key(i);
                        }
                    });
                    insert!(&db, WayInsert)
                        .transaction(&mut tx)
                        .bulk(ways.values())
                        .await
                        .expect("Error while inserting ways");

                    insert!(&db, NodeInsert)
                        .transaction(&mut tx)
                        .bulk(nodes.values())
                        .await
                        .expect("Error while inserting ways");

                    insert!(&db, AreaInsert)
                        .transaction(&mut tx)
                        .bulk(areas.values())
                        .await
                        .expect("Error while inserting ways");

                    tx.commit().await.expect("Could not commit transaction");
                })
                .await;

            println!("DB Insert: {}", Utc::now().sub(after_osm));

            Ok(())
        }
    }
}
