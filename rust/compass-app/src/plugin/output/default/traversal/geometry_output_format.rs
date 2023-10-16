use super::traversal_ops as ops;
use crate::{app::search::search_app_result::SearchAppResult, plugin::plugin_error::PluginError};
use geo::LineString;
use serde::{Deserialize, Serialize};
use wkt::ToWkt;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum GeometryOutputFormat {
    AggregateWkt,
    EdgePropertiesJson,
    GeoJson,
}

impl GeometryOutputFormat {
    pub fn generate_route_output(
        &self,
        result: &SearchAppResult,
        geoms: &Vec<LineString<f64>>,
    ) -> Result<serde_json::Value, PluginError> {
        match self {
            GeometryOutputFormat::AggregateWkt => {
                let route_geometry = ops::create_route_geometry(&result.route, geoms)?;
                let route_wkt = route_geometry.wkt_string();
                Ok(serde_json::Value::String(route_wkt))
            }
            GeometryOutputFormat::EdgePropertiesJson => {
                let result = serde_json::to_value(&result.route)?;
                Ok(result)
            }
            GeometryOutputFormat::GeoJson => {
                let result = ops::create_annotated_with_geometries(&result.route, geoms)?;
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
    use compass_core::{
        algorithm::search::edge_traversal::EdgeTraversal,
        model::{
            cost::cost::Cost, graph::edge_id::EdgeId, traversal::state::state_variable::StateVar,
        },
    };
    use geo::{coord, LineString};
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
        ];

        println!(
            "{:?}",
            GeometryOutputFormat::AggregateWkt
                .generate_route_output(&result, &geoms)
                .map(|r| serde_json::to_string_pretty(&r))
        );
        println!(
            "{:?}",
            GeometryOutputFormat::EdgePropertiesJson
                .generate_route_output(&result, &geoms)
                .map(|r| serde_json::to_string_pretty(&r))
        );
        println!(
            "{:?}",
            GeometryOutputFormat::GeoJson
                .generate_route_output(&result, &geoms)
                .map(|r| serde_json::to_string_pretty(&r))
        );
    }
}
