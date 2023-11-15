use std::collections::HashMap;

use super::traversal_ops as ops;
use crate::plugin::plugin_error::PluginError;
use geo::LineString;
use routee_compass_core::{
    algorithm::search::{edge_traversal::EdgeTraversal, search_tree_branch::SearchTreeBranch},
    model::road_network::vertex_id::VertexId,
};
use serde::{Deserialize, Serialize};
use wkt::ToWkt;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum TraversalOutputFormat {
    // concatenates all LINESTRINGS and returns the geometry as a WKT
    Wkt,
    // returns the properties of each link traversal as a JSON array of objects
    Json,
    // returns the geometries and properties as GeoJSON
    GeoJson,
}

impl TraversalOutputFormat {
    /// generates output for a route based on the configured TraversalOutputFormat
    pub fn generate_route_output(
        &self,
        route: &Vec<EdgeTraversal>,
        geoms: &[LineString<f64>],
    ) -> Result<serde_json::Value, PluginError> {
        match self {
            TraversalOutputFormat::Wkt => {
                let route_geometry = ops::create_route_linestring(route, geoms)?;
                let route_wkt = route_geometry.wkt_string();
                Ok(serde_json::Value::String(route_wkt))
            }
            TraversalOutputFormat::Json => {
                let result = serde_json::to_value(route)?;
                Ok(result)
            }
            TraversalOutputFormat::GeoJson => {
                let result = ops::create_route_geojson(route, geoms)?;
                Ok(result)
            }
        }
    }

    /// generates output for a tree based on the configured TraversalOutputFormat
    pub fn generate_tree_output(
        &self,
        tree: &HashMap<VertexId, SearchTreeBranch>,
        geoms: &[LineString<f64>],
    ) -> Result<serde_json::Value, PluginError> {
        match self {
            TraversalOutputFormat::Wkt => {
                let route_geometry = ops::create_tree_multilinestring(tree, geoms)?;
                let route_wkt = route_geometry.wkt_string();
                Ok(serde_json::Value::String(route_wkt))
            }
            TraversalOutputFormat::Json => {
                let result = serde_json::to_value(tree.values().collect::<Vec<_>>())?;
                Ok(result)
            }
            TraversalOutputFormat::GeoJson => {
                let result = ops::create_tree_geojson(tree, geoms)?;
                Ok(result)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::app::search::search_app_result::SearchAppResult;
    use chrono::Local;
    use geo::{coord, LineString};
    use routee_compass_core::{
        algorithm::search::edge_traversal::EdgeTraversal,
        model::{
            cost::Cost, road_network::edge_id::EdgeId, traversal::state::state_variable::StateVar,
        },
    };
    use std::{collections::HashMap, time::Duration};

    #[test]
    fn test() {
        let route = vec![
            EdgeTraversal {
                edge_id: EdgeId(0),
                access_cost: Cost::from(0.0),
                traversal_cost: Cost::from(10.0),
                result_state: vec![StateVar(10.0)],
            },
            EdgeTraversal {
                edge_id: EdgeId(1),
                access_cost: Cost::from(5.0),
                traversal_cost: Cost::from(9.0),
                result_state: vec![StateVar(24.0)],
            },
            EdgeTraversal {
                edge_id: EdgeId(2),
                access_cost: Cost::from(0.0),
                traversal_cost: Cost::from(11.0),
                result_state: vec![StateVar(35.0)],
            },
        ];
        let result = SearchAppResult {
            route,
            tree: HashMap::new(),
            search_start_time: Local::now(),
            search_runtime: Duration::ZERO,
            route_runtime: Duration::ZERO,
            total_runtime: Duration::ZERO,
        };

        let geoms = vec![
            LineString(vec![
                coord! { x: 1.0, y: 0.0 },
                coord! { x: 1.0, y: 0.0 },
                coord! { x: 1.0, y: 1.0 },
            ]),
            LineString(vec![coord! { x: 2.0, y: 2.0 }, coord! { x: 2.0, y: 3.0 }]),
            LineString(vec![coord! { x: 3.0, y: 3.0 }, coord! { x: 3.0, y: 4.0 }]),
        ]
        .into_boxed_slice();

        println!(
            "{:?}",
            TraversalOutputFormat::Wkt
                .generate_route_output(&result.route, &geoms)
                .map(|r| serde_json::to_string_pretty(&r))
        );
        println!(
            "{:?}",
            TraversalOutputFormat::Json
                .generate_route_output(&result.route, &geoms)
                .map(|r| serde_json::to_string_pretty(&r))
        );
        println!(
            "{:?}",
            TraversalOutputFormat::GeoJson
                .generate_route_output(&result.route, &geoms)
                .map(|r| serde_json::to_string_pretty(&r))
        );
    }
}
