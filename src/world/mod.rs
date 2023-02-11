use std::collections::{HashMap, HashSet};

use linear_map::LinearMap;
use log::info;
use rorm::conditions::Condition;
use rorm::Model;
use rorm::{and, query, Database};
use rustymon_world::geometry::{polygon, polyline, Point};
use rustymon_world::projection::{Projection, Simple};
use serde::Deserialize;

use crate::models::db::{Area, Node, Tile, Way};

pub static TAGS_FILE: &'static str = include_str!("../../data/spawns.json");

pub struct OSMTags(Vec<(&'static str, Vec<&'static str>)>);
impl Default for OSMTags {
    /// Create a new instance by parsing the bundled file
    fn default() -> Self {
        Self::new()
    }
}
impl OSMTags {
    /// Create a new instance by parsing the bundled file
    pub fn new() -> Self {
        let map: LinearMap<&'static str, Vec<&'static str>> =
            serde_json::from_str(TAGS_FILE).expect("spawns.json should be valid");
        Self(map.into_iter().collect())
    }

    /// Convert a list of key-value arrays into a key-values map
    pub fn lookup(
        &self,
        tags: impl Iterator<Item = [u32; 2]>,
    ) -> Option<HashMap<&'static str, Vec<&'static str>>> {
        let mut result = HashMap::new();
        for [key, value] in tags {
            let &(key, ref values) = self.0.get(key as usize)?;
            let &value = values.get(value as usize)?;

            result.entry(key).or_insert(Vec::new()).push(value);
        }
        Some(result)
    }
}

#[derive(Deserialize)]
pub struct Coord {
    pub lat: f64,
    pub lng: f64,
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

const NODE_DISTANCE: f64 = 1.0;
const WAY_DISTANCE: f64 = 1.0;

pub unsafe fn slice_from_bytes<T>(bytes: &[u8]) -> &[T] {
    std::slice::from_raw_parts(
        bytes.as_ptr() as *const T,
        bytes.len() / std::mem::size_of::<T>(),
    )
}
pub unsafe fn bytes_from_slice<T>(bytes: &[T]) -> &[u8] {
    std::slice::from_raw_parts(
        bytes.as_ptr() as *const u8,
        bytes.len() * std::mem::size_of::<T>(),
    )
}

pub async fn get_osm_tags(db: &Database, coord: &Coord) -> Result<HashSet<[u32; 2]>, rorm::Error> {
    let point = Simple.project_nalgebra(Point::new(coord.lng, coord.lat));

    let tiles = query!(&db, Tile)
        .condition(tile_condition(point))
        .all()
        .await?;

    let mut tags = HashSet::new();
    for tile in tiles {
        for area in query!(&db, Area)
            .condition(Area::F.tile.equals(tile.id))
            .all()
            .await?
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
            .await?
        {
            if point.metric_distance(&Point::new(node.x, node.y)) < NODE_DISTANCE {
                let features = unsafe { slice_from_bytes::<[u32; 2]>(&node.features) };
                tags.extend(features.iter().copied());
            }
        }

        for way in query!(&db, Way)
            .condition(Way::F.tile.equals(tile.id))
            .all()
            .await?
        {
            let points = unsafe { slice_from_bytes(&way.points) };
            if polyline::distance_to(points, point) < WAY_DISTANCE {
                let features = unsafe { slice_from_bytes::<[u32; 2]>(&way.features) };
                tags.extend(features.iter().copied());
            }
        }
    }

    Ok(tags)
}
