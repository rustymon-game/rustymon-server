use rorm::{Model, Patch};

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
