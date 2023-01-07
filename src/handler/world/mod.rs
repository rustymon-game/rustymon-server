use actix_web::web::{Data, Json, Query};
use rorm::conditions::Condition;
use rorm::{and, query, Database, Model};
use rustymon_world::geometry::{polygon, polyline, Point};
use rustymon_world::projection::{Projection, WebMercator};
use serde::Deserialize;
use std::collections::HashSet;

use crate::models::db::{Area, Node, Tile, Way};

#[derive(Deserialize)]
pub struct Coord {
    lat: f64,
    lng: f64,
}

/// Convert a point into a condition which can be used to query the tile containing the point
pub fn tile_condition<'a>(point: Point) -> impl Condition<'a> {
    and!(
        Tile::F.min_x.less_or_equals(point.x),
        Tile::F.max_x.greater_or_equals(point.x),
        Tile::F.min_y.less_or_equals(point.y),
        Tile::F.max_y.greater_or_equals(point.y)
    )
}

const NODE_DISTANCE: f64 = 0.0;
const WAY_DISTANCE: f64 = 0.0;

unsafe fn slice_from_bytes<T>(bytes: &[u8]) -> &[T] {
    std::slice::from_raw_parts(
        bytes.as_ptr() as *const T,
        bytes.len() / std::mem::size_of::<T>(),
    )
}

/// Return a list of osm tags for a given location
pub async fn get_tags(db: Data<Database>, coord: Query<Coord>) -> Json<HashSet<[u32; 2]>> {
    let point = WebMercator.project_nalgebra(Point::new(coord.lng, coord.lat));

    let tiles = query!(&db, Tile)
        .condition(tile_condition(point))
        .all()
        .await
        .unwrap();

    let mut tags = HashSet::new();
    for tile in tiles {
        for area in query!(&db, Area)
            .condition(Area::F.tile.equals(tile.id))
            .all()
            .await
            .unwrap()
        {
            let points = unsafe { slice_from_bytes(&area.points) };
            if polygon::contains_point(points, point) {
                let features = unsafe { slice_from_bytes::<[u32; 2]>(&area.features) };
                tags.extend(features.iter().copied());
            }
        }

        for node in query!(&db, Node)
            .condition(Node::F.tile.equals(tile.id))
            .all()
            .await
            .unwrap()
        {
            if point.metric_distance(&Point::new(node.x, node.y)) < NODE_DISTANCE {
                let features = unsafe { slice_from_bytes::<[u32; 2]>(&node.features) };
                tags.extend(features.iter().copied());
            }
        }

        for way in query!(&db, Way)
            .condition(Way::F.tile.equals(tile.id))
            .all()
            .await
            .unwrap()
        {
            let points = unsafe { slice_from_bytes(&way.points) };
            if polyline::distance_to(points, point) < WAY_DISTANCE {
                let features = unsafe { slice_from_bytes::<[u32; 2]>(&way.features) };
                tags.extend(features.iter().copied());
            }
        }
    }

    Json(tags)
}
