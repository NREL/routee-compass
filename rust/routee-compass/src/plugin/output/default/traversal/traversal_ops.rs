use crate::plugin::output::OutputPluginError;
use geo::{LineString, MultiLineString, Point};
use geo_types::MultiPoint;
use geojson::feature::Id;
use geojson::{Feature, FeatureCollection};
use routee_compass_core::algorithm::search::EdgeTraversal;
use routee_compass_core::algorithm::search::SearchTreeBranch;
use routee_compass_core::model::map::MapModel;
use routee_compass_core::model::network::vertex_id::VertexId;
use routee_compass_core::util::geo::geo_io_utils;
use std::collections::HashMap;
use std::sync::Arc;

pub fn create_tree_geojson(
    tree: &HashMap<VertexId, SearchTreeBranch>,
    map_model: Arc<MapModel>,
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
                .and_then(|g| create_geojson_feature(&t.edge_traversal, g));

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
) -> Result<serde_json::Value, OutputPluginError> {
    let features = route
        .iter()
        .map(|t| {
            let row_result = map_model
                .get(&t.edge_id)
                .cloned()
                .map_err(|e| {
                    OutputPluginError::OutputPluginFailed(format!(
                        "failure building route geojson: {}",
                        e
                    ))
                })
                .and_then(|g| create_geojson_feature(t, g));

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

pub fn create_geojson_feature(
    t: &EdgeTraversal,
    g: LineString<f32>,
) -> Result<Feature, OutputPluginError> {
    let props = match serde_json::to_value(t).map(|v| v.as_object().cloned()) {
        Ok(None) => Err(OutputPluginError::InternalError(format!(
            "serialized EdgeTraversal was not a JSON object for {}",
            t
        ))),
        Ok(Some(obj)) => Ok(obj),
        Err(err) => Err(OutputPluginError::JsonError { source: err }),
    }?;

    let id = Id::Number(serde_json::Number::from(t.edge_id.0));
    let geometry = geojson::Geometry::from(&g);
    let feature = Feature {
        bbox: None,
        geometry: Some(geometry),
        id: Some(id),
        properties: Some(props),
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
    tree: &HashMap<VertexId, SearchTreeBranch>,
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
                    l.points().last().ok_or_else(|| {
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
