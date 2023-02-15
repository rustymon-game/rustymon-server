use rorm::{ForeignModel, Model, Patch};
use rustymon_world::features::{prototyping, FeatureParser};
use rustymon_world::formats;
use rustymon_world::geometry::Point;
use serde::{Deserialize, Serialize};

type ParsedArea<'tile> =
    formats::Item<&'tile <prototyping::Parser as FeatureParser>::Feature, &'tile [Point]>;
type ParsedNode<'tile> =
    formats::Item<&'tile <prototyping::Parser as FeatureParser>::Feature, &'tile Point>;
type ParsedWay<'tile> =
    formats::Item<&'tile <prototyping::Parser as FeatureParser>::Feature, &'tile [Point]>;

#[derive(Model, Serialize, Deserialize)]
pub(crate) struct User {
    #[rorm(primary_key, max_length = 255)]
    pub(crate) username: String,
    #[rorm(max_length = 255)]
    pub(crate) display_name: String,
    #[rorm(max_length = 1024)]
    pub(crate) password_hash: String,

    #[rorm(auto_create_time)]
    pub(crate) created_at: chrono::NaiveDateTime,
}

#[derive(Model)]
pub(crate) struct Tile {
    #[rorm(id)]
    pub(crate) id: i64,

    pub(crate) min_x: f64,
    pub(crate) max_x: f64,
    pub(crate) min_y: f64,
    pub(crate) max_y: f64,
}

#[derive(Patch)]
#[rorm(model = "Tile")]
pub(crate) struct TileInsert {
    pub(crate) min_x: f64,
    pub(crate) max_x: f64,
    pub(crate) min_y: f64,
    pub(crate) max_y: f64,
}

#[derive(Model)]
pub(crate) struct Way {
    #[rorm(id)]
    pub(crate) id: i64,

    #[rorm(on_update = "Cascade")]
    pub(crate) tile: ForeignModel<Tile>,

    points: Vec<u8>,
    features: Vec<u8>,
}

#[derive(Patch)]
#[rorm(model = "Way")]
pub(crate) struct WayInsert {
    pub(crate) tile: ForeignModel<Tile>,
    points: Vec<u8>,
    features: Vec<u8>,
}
impl WayInsert {
    pub(crate) fn new(tile: i64, way: ParsedWay) -> Self {
        unsafe {
            Self {
                tile: ForeignModel::Key(tile),
                points: bytes_from_slice(way.points).to_vec(),
                features: bytes_from_slice(way.feature).to_vec(),
            }
        }
    }
}

#[derive(Model)]
pub(crate) struct Area {
    #[rorm(id)]
    pub(crate) id: i64,

    #[rorm(on_update = "Cascade")]
    pub(crate) tile: ForeignModel<Tile>,
    points: Vec<u8>,
    features: Vec<u8>,
}

#[derive(Patch)]
#[rorm(model = "Area")]
pub(crate) struct AreaInsert {
    pub(crate) tile: ForeignModel<Tile>,
    points: Vec<u8>,
    features: Vec<u8>,
}
impl AreaInsert {
    pub(crate) fn new(tile: i64, area: ParsedArea) -> Self {
        unsafe {
            Self {
                tile: ForeignModel::Key(tile),
                points: bytes_from_slice(area.points).to_vec(),
                features: bytes_from_slice(area.feature).to_vec(),
            }
        }
    }
}

#[derive(Model)]
pub(crate) struct Node {
    #[rorm(id)]
    pub(crate) id: i64,

    #[rorm(on_update = "Cascade")]
    pub(crate) tile: ForeignModel<Tile>,

    pub(crate) x: f64,
    pub(crate) y: f64,

    features: Vec<u8>,
}

#[derive(Patch)]
#[rorm(model = "Node")]
pub(crate) struct NodeInsert {
    pub(crate) tile: ForeignModel<Tile>,
    pub(crate) x: f64,
    pub(crate) y: f64,
    features: Vec<u8>,
}
impl NodeInsert {
    pub(crate) fn new(tile: i64, node: ParsedNode) -> Self {
        unsafe {
            Self {
                tile: ForeignModel::Key(tile),
                x: node.points.x,
                y: node.points.y,
                features: bytes_from_slice(node.feature).to_vec(),
            }
        }
    }
}

macro_rules! impl_features_getter {
    ($($strct:ty),*) => {
        $(
            impl $strct {
                 pub(crate) fn features(&self) -> &[[u32; 2]] {
                    unsafe { slice_from_bytes(&self.features) }
                 }
            }
        )*
    }
}
impl_features_getter![Area, Way, Node];

macro_rules! impl_points_getter {
    ($($strct:ty),*) => {
        $(
            impl $strct {
                 pub(crate) fn points(&self) -> &[Point] {
                    unsafe { slice_from_bytes(&self.points) }
                }
            }
        )*
    }
}
impl_points_getter![Area, Way];

unsafe fn slice_from_bytes<T>(bytes: &[u8]) -> &[T] {
    std::slice::from_raw_parts(
        bytes.as_ptr() as *const T,
        bytes.len() / std::mem::size_of::<T>(),
    )
}
unsafe fn bytes_from_slice<T>(bytes: &[T]) -> &[u8] {
    std::slice::from_raw_parts(
        bytes.as_ptr() as *const u8,
        bytes.len() * std::mem::size_of::<T>(),
    )
}
