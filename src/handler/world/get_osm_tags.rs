use std::collections::HashMap;

use actix_web::web::{Data, Json, Query};
use rorm::Database;

use crate::world::{self, Coord, OSMTags};

pub async fn get_osm_tags(
    db: Data<Database>,
    tags: Data<OSMTags>,
    coord: Query<Coord>,
) -> Json<HashMap<&'static str, Vec<&'static str>>> {
    Json(
        tags.lookup(world::get_osm_tags(&db, &coord).await.unwrap().into_iter())
            .unwrap(),
    )
}
