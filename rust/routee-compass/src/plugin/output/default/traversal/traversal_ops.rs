use crate::plugin::plugin_error::PluginError;
use geo::{LineString, MultiLineString, Point};
use geo_types::MultiPoint;
use geojson::feature::Id;
use geojson::{Feature, FeatureCollection};
use routee_compass_core::algorithm::search::edge_traversal::EdgeTraversal;
use routee_compass_core::algorithm::search::search_tree_branch::SearchTreeBranch;
use routee_compass_core::model::network::vertex_id::VertexId;
use routee_compass_core::util::geo::geo_io_utils;
use std::collections::HashMap;

pub fn create_tree_geojson(
    tree: &HashMap<VertexId, SearchTreeBranch>,
    geoms: &[LineString<f32>],
) -> Result<serde_json::Value, PluginError> {
    let features = tree
        .values()
        .map(|t| {
            let row_result = geoms
                .get(t.edge_traversal.edge_id.0)
                .cloned()
                .ok_or_else(|| PluginError::EdgeGeometryMissing(t.edge_traversal.edge_id))
                .and_then(|g| create_geojson_feature(&t.edge_traversal, g));

            row_result
        })
        .collect::<Result<Vec<_>, PluginError>>()?;
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
    geoms: &[LineString<f32>],
) -> Result<serde_json::Value, PluginError> {
    let features = route
        .iter()
        .map(|t| {
            let row_result = geoms
                .get(t.edge_id.0)
                .cloned()
                .ok_or_else(|| PluginError::EdgeGeometryMissing(t.edge_id))
                .and_then(|g| create_geojson_feature(t, g));

            row_result
        })
        .collect::<Result<Vec<_>, PluginError>>()?;
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
) -> Result<Feature, PluginError> {
    let props = match serde_json::to_value(t).map(|v| v.as_object().cloned()) {
        Ok(None) => Err(PluginError::InternalError(format!(
            "serialized EdgeTraversal was not a JSON object for {}",
            t
        ))),
        Ok(Some(obj)) => Ok(obj),
        Err(err) => Err(PluginError::JsonError(err)),
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
) -> Result<LineString<f32>, PluginError> {
    geoms
        .get(edge.edge_id.0)
        .cloned()
        .ok_or_else(|| PluginError::EdgeGeometryMissing(edge.edge_id))
}

pub fn create_branch_geometry(
    branch: &SearchTreeBranch,
    geoms: &[LineString<f32>],
) -> Result<LineString<f32>, PluginError> {
    create_edge_geometry(&branch.edge_traversal, geoms)
}

pub fn create_route_linestring(
    route: &[EdgeTraversal],
    geoms: &[LineString<f32>],
) -> Result<LineString<f32>, PluginError> {
    let edge_ids = route
        .iter()
        .map(|traversal| traversal.edge_id)
        .collect::<Vec<_>>();

    let edge_linestrings = edge_ids
        .iter()
        .map(|eid| {
            let geom = geoms
                .get(eid.0)
                .ok_or_else(|| PluginError::EdgeGeometryMissing(*eid));
            geom
        })
        .collect::<Result<Vec<&LineString<f32>>, PluginError>>()?;
    let geometry = geo_io_utils::concat_linestrings(edge_linestrings);
    Ok(geometry)
}

pub fn create_tree_multilinestring(
    tree: &HashMap<VertexId, SearchTreeBranch>,
    geoms: &[LineString<f32>],
) -> Result<MultiLineString<f32>, PluginError> {
    let edge_ids = tree
        .values()
        .map(|traversal| traversal.edge_traversal.edge_id)
        .collect::<Vec<_>>();

    let tree_linestrings = edge_ids
        .iter()
        .map(|eid| {
            let geom = geoms
                .get(eid.0)
                .ok_or_else(|| PluginError::EdgeGeometryMissing(*eid));
            geom.cloned()
        })
        .collect::<Result<Vec<LineString<f32>>, PluginError>>()?;
    let geometry = MultiLineString::new(tree_linestrings);
    Ok(geometry)
}

pub fn create_tree_multipoint(
    tree: &HashMap<VertexId, SearchTreeBranch>,
    geoms: &[LineString<f64>],
) -> Result<MultiPoint, PluginError> {
    let edge_ids = tree
        .values()
        .map(|traversal| traversal.edge_traversal.edge_id)
        .collect::<Vec<_>>();

    let tree_destinations = edge_ids
        .iter()
        .map(|eid| {
            let geom = geoms
                .get(eid.0)
                .ok_or_else(|| PluginError::EdgeGeometryMissing(*eid))
                .map(|l| {
                    l.points().last().ok_or_else(|| {
                        PluginError::InputError(format!(
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
        .collect::<Result<Vec<Point>, PluginError>>()?;
    let geometry = MultiPoint::new(tree_destinations);
    Ok(geometry)
}
