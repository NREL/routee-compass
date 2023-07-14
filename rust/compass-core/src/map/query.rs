use serde::{Serialize, Deserialize};

use crate::model::graph::vertex_id::VertexId;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserQuery {
    pub origin_latitude: f64,
    pub origin_longitude: f64,
    pub destination_latitude: f64,
    pub destination_longitude: f64,

    pub origin_vertex: Option<VertexId>,
    pub destination_vertex: Option<VertexId>,
}