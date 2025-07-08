use crate::plugin::output::OutputPluginError;
use geo::{LineString, MultiLineString, Point};
use geo_types::MultiPoint;
use geojson::{Feature, FeatureCollection};
use routee_compass_core::algorithm::search::EdgeTraversal;
use routee_compass_core::algorithm::search::SearchTreeBranch;
use routee_compass_core::model::cost::CostModel;
use routee_compass_core::model::label::Label;
use routee_compass_core::model::map::MapModel;
use routee_compass_core::model::network::vertex_id::VertexId;
use routee_compass_core::model::state::StateModel;
use routee_compass_core::util::geo::geo_io_utils;
use serde_json::{json, Map};
use std::collections::HashMap;
use std::sync::Arc;

pub fn create_tree_geojson(
    tree: &HashMap<Label, SearchTreeBranch>,
    map_model: Arc<MapModel>,
    state_model: Arc<StateModel>,
    cost_model: Arc<CostModel>,
) -> Result<serde_json::Value, OutputPluginError> {
    let features = tree
        .values()
        .map(|t| {
            let row_result = map_model
                .get(&t.edge_traversal.edge_id)
                .cloned()
                .map_err(|e| {
                    OutputPluginError::OutputPluginFailed(format!(
                        "failure creating tree GeoJSON: {}",
                        e
                    ))
                })
                .and_then(|g| {
                    create_geojson_feature(
                        &t.edge_traversal,
                        g,
                        state_model.clone(),
                        cost_model.clone(),
                    )
                });

            row_result
        })
        .collect::<Result<Vec<_>, OutputPluginError>>()?;
    // let result_json = serde_json::to_value(features)?;/
    let feature_collection = FeatureCollection {
        bbox: None,
        features,
        foreign_members: None,
    };
    let result = serde_json::to_value(feature_collection)?;
    Ok(result)
}

pub fn create_route_geojson(
    route: &[EdgeTraversal],
    map_model: Arc<MapModel>,
    state_model: Arc<StateModel>,
    cost_model: Arc<CostModel>,
) -> Result<serde_json::Value, OutputPluginError> {
    let features = route
        .iter()
        .map(|t| {
            let g = map_model.get(&t.edge_id).cloned().map_err(|e| {
                OutputPluginError::OutputPluginFailed(format!(
                    "failure building route geojson: {}",
                    e
                ))
            })?;
            let geojson_feature =
                create_geojson_feature(t, g, state_model.clone(), cost_model.clone())?;
            Ok(geojson_feature)
        })
        .collect::<Result<Vec<_>, OutputPluginError>>()?;
    // let result_json = serde_json::to_value(features)?;/
    let feature_collection = FeatureCollection {
        bbox: None,
        features,
        foreign_members: None,
    };
    let result = serde_json::to_value(feature_collection)?;
    Ok(result)
}

pub fn create_geojson_feature(
    t: &EdgeTraversal,
    g: LineString<f32>,
    state_model: Arc<StateModel>,
    cost_model: Arc<CostModel>,
) -> Result<Feature, OutputPluginError> {
    let serialized_state = state_model.serialize_state(&t.result_state);
    let serialized_cost = cost_model
        .serialize_cost(&t.result_state, state_model.clone())
        .map_err(|e| {
            OutputPluginError::OutputPluginFailed(format!(
                "failure serializing cost for geojson feature: {}",
                e
            ))
        })?;

    let serialized_traversal = match serde_json::to_value(t).map(|v| v.as_object().cloned()) {
        Ok(Some(obj)) => Ok(json![obj]),
        Ok(None) => Err(OutputPluginError::InternalError(format!(
            "serialized EdgeTraversal was not a JSON object for {}",
            t
        ))),
        Err(err) => Err(OutputPluginError::JsonError { source: err }),
    }?;
    let mut properties = Map::new();
    properties.insert(String::from("edge_id"), json![t.edge_id]);
    properties.insert(String::from("traversal"), serialized_traversal);
    properties.insert(String::from("state"), serialized_state);
    properties.insert(String::from("cost"), serialized_cost);

    // let id = Id::Number(serde_json::Number::from(t.edge_id.0));
    let geometry = geojson::Geometry::from(&g);
    let feature = Feature {
        bbox: None,
        geometry: Some(geometry),
        id: None,
        properties: Some(properties),
        foreign_members: None,
    };
    Ok(feature)
}

pub fn create_edge_geometry(
    edge: &EdgeTraversal,
    geoms: &[LineString<f32>],
) -> Result<LineString<f32>, OutputPluginError> {
    geoms.get(edge.edge_id.0).cloned().ok_or_else(|| {
        OutputPluginError::OutputPluginFailed(format!(
            "geometry table missing edge id {}",
            edge.edge_id
        ))
    })
}

pub fn create_branch_geometry(
    branch: &SearchTreeBranch,
    geoms: &[LineString<f32>],
) -> Result<LineString<f32>, OutputPluginError> {
    create_edge_geometry(&branch.edge_traversal, geoms)
}

pub fn create_route_linestring(
    route: &[EdgeTraversal],
    map_model: Arc<MapModel>,
) -> Result<LineString<f32>, OutputPluginError> {
    let edge_ids = route
        .iter()
        .map(|traversal| traversal.edge_id)
        .collect::<Vec<_>>();

    let edge_linestrings = edge_ids
        .iter()
        .map(|eid| {
            let geom = map_model.get(eid).map_err(|e| {
                OutputPluginError::OutputPluginFailed(format!(
                    "failure building route linestring: {}",
                    e
                ))
            });
            geom
        })
        .collect::<Result<Vec<&LineString<f32>>, OutputPluginError>>()?;
    let geometry = geo_io_utils::concat_linestrings(edge_linestrings);
    Ok(geometry)
}

pub fn create_tree_multilinestring(
    tree: &HashMap<Label, SearchTreeBranch>,
    // geoms: &[LineString<f32>],
    map_model: Arc<MapModel>,
) -> Result<MultiLineString<f32>, OutputPluginError> {
    let edge_ids = tree
        .values()
        .map(|traversal| traversal.edge_traversal.edge_id)
        .collect::<Vec<_>>();

    let tree_linestrings = edge_ids
        .iter()
        .map(|eid| {
            let geom = map_model.get(eid).map_err(|e| {
                OutputPluginError::OutputPluginFailed(format!("failure building tree WKT: {}", e))
            });
            geom.cloned()
        })
        .collect::<Result<Vec<LineString<f32>>, OutputPluginError>>()?;
    let geometry = MultiLineString::new(tree_linestrings);
    Ok(geometry)
}

pub fn create_tree_multipoint(
    tree: &HashMap<VertexId, SearchTreeBranch>,
    geoms: &[LineString<f64>],
) -> Result<MultiPoint, OutputPluginError> {
    let edge_ids = tree
        .values()
        .map(|traversal| traversal.edge_traversal.edge_id)
        .collect::<Vec<_>>();

    let tree_destinations = edge_ids
        .iter()
        .map(|eid| {
            let geom = geoms
                .get(eid.0)
                .ok_or_else(|| {
                    OutputPluginError::OutputPluginFailed(format!(
                        "geometry table missing edge id {}",
                        *eid
                    ))
                })
                .map(|l| {
                    l.points().next_back().ok_or_else(|| {
                        OutputPluginError::OutputPluginFailed(format!(
                            "linestring is invalid for edge_id {}",
                            eid
                        ))
                    })
                });
            match geom {
                // rough "result flatten"
                Ok(Ok(p)) => Ok(p),
                Ok(Err(e)) => Err(e),
                Err(e) => Err(e),
            }
        })
        .collect::<Result<Vec<Point>, OutputPluginError>>()?;
    let geometry = MultiPoint::new(tree_destinations);
    Ok(geometry)
}
