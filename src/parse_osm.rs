use std::collections::HashMap;
use std::slice;

use rorm::internal::field::foreign_model::ForeignModelByField;
use rorm::{insert, Database};
use rustymon_world::features::prototyping;

use crate::models::db::{AreaInsert, NodeInsert, TileInsert, WayInsert};

const SPAWNS: &str = include_str!("../data/spawns.json");

pub(crate) async fn parse_osm(
    db: Database,
    file: String,
    cols: usize,
    rows: usize,
    center_x: f64,
    center_y: f64,
    zoom: u8,
) -> Result<(), String> {
    let osm_tiles = rustymon_world::parse(rustymon_world::Config {
        zoom,
        center_x,
        center_y,
        rows,
        cols,
        file,
        visual: prototyping::Parser::from_file(SPAWNS).unwrap(),
    })?;

    let mut tiles = HashMap::new();
    let mut ways = HashMap::new();
    let mut nodes = HashMap::new();
    let mut areas = HashMap::new();

    osm_tiles.iter().enumerate().for_each(|(idx, t)| {
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
                        slice::from_raw_parts(a.points.as_ptr() as *const u8, a.points.len())
                    }
                    .to_vec(),
                    features: unsafe {
                        slice::from_raw_parts(a.feature.as_ptr() as *const u8, a.feature.len())
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
                        slice::from_raw_parts(n.feature.as_ptr() as *const u8, n.feature.len())
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
                        slice::from_raw_parts(w.points.as_ptr() as *const u8, w.points.len())
                    }
                    .to_vec(),
                    features: unsafe {
                        slice::from_raw_parts(w.feature.as_ptr() as *const u8, w.feature.len())
                    }
                    .to_vec(),
                },
            );
        });
    });

    let tile_ids = insert!(&db, TileInsert)
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

    let way_fut = async {
        insert!(&db, WayInsert)
            .bulk(ways.values())
            .await
            .expect("Error while inserting ways");
    };

    let node_fut = async {
        insert!(&db, NodeInsert)
            .bulk(nodes.values())
            .await
            .expect("Error while inserting ways");
    };

    let area_fut = async {
        insert!(&db, AreaInsert)
            .bulk(areas.values())
            .await
            .expect("Error while inserting ways");
    };

    futures::join!(way_fut, node_fut, area_fut);

    Ok(())
}
