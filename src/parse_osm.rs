use rorm::{insert, Database};
use rustymon_world::features::prototyping;

use crate::models::db::{AreaInsert, NodeInsert, TileInsert, WayInsert};
use crate::world::{PROJECTION, TAGS_FILE, ZOOM};

pub(crate) async fn parse_osm(
    db: Database,
    file: String,
    cols: usize,
    rows: usize,
    center_x: f64,
    center_y: f64,
) -> Result<(), String> {
    let osm_tiles = rustymon_world::parse(rustymon_world::Config {
        zoom: ZOOM,
        center_x,
        center_y,
        rows,
        cols,
        file,
        visual: prototyping::Parser::from_file(TAGS_FILE).unwrap(),
        projection: PROJECTION,
    })?;

    let tiles = Vec::from_iter(osm_tiles.iter().map(|tile| TileInsert {
        min_x: tile.min.x,
        min_y: tile.min.y,
        max_x: tile.max.x,
        max_y: tile.max.y,
    }));
    let tiles = insert!(&db, TileInsert)
        .bulk(&tiles)
        .await
        .expect("Error while creating tiles");

    let mut ways = Vec::new();
    let mut nodes = Vec::new();
    let mut areas = Vec::new();
    for (tile, &id) in osm_tiles.iter().zip(tiles.iter()) {
        for area in tile.iter_areas() {
            areas.push(AreaInsert::new(id, area));
        }

        for node in tile.iter_nodes() {
            nodes.push(NodeInsert::new(id, node));
        }

        for way in tile.iter_ways() {
            ways.push(WayInsert::new(id, way));
        }
    }

    let way_fut = async {
        insert!(&db, WayInsert)
            .bulk(&ways)
            .await
            .expect("Error while inserting ways");
    };

    let node_fut = async {
        insert!(&db, NodeInsert)
            .bulk(&nodes)
            .await
            .expect("Error while inserting ways");
    };

    let area_fut = async {
        insert!(&db, AreaInsert)
            .bulk(&areas)
            .await
            .expect("Error while inserting ways");
    };

    futures::join!(way_fut, node_fut, area_fut);

    Ok(())
}
