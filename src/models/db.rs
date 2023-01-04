use rorm::{ForeignModel, Model, Patch};
use serde::{Deserialize, Serialize};

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

    pub(crate) points: Vec<u8>,
    pub(crate) features: Vec<u8>,
}

#[derive(Patch)]
#[rorm(model = "Way")]
pub(crate) struct WayInsert {
    pub(crate) tile: ForeignModel<Tile>,
    pub(crate) points: Vec<u8>,
    pub(crate) features: Vec<u8>,
}

#[derive(Model)]
pub(crate) struct Area {
    #[rorm(id)]
    pub(crate) id: i64,

    #[rorm(on_update = "Cascade")]
    pub(crate) tile: ForeignModel<Tile>,

    pub(crate) points: Vec<u8>,
    pub(crate) features: Vec<u8>,
}

#[derive(Patch)]
#[rorm(model = "Area")]
pub(crate) struct AreaInsert {
    pub(crate) tile: ForeignModel<Tile>,
    pub(crate) points: Vec<u8>,
    pub(crate) features: Vec<u8>,
}

#[derive(Model)]
pub(crate) struct Node {
    #[rorm(id)]
    pub(crate) id: i64,

    #[rorm(on_update = "Cascade")]
    pub(crate) tile: ForeignModel<Tile>,

    pub(crate) x: f64,
    pub(crate) y: f64,

    pub(crate) features: Vec<u8>,
}

#[derive(Patch)]
#[rorm(model = "Node")]
pub(crate) struct NodeInsert {
    pub(crate) tile: ForeignModel<Tile>,
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) features: Vec<u8>,
}
