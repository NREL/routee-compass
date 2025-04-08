//! Configuration for building an instance of an [`super::InjectInputPlugin`].

use super::{inject_plugin::InjectInputPlugin, CoordinateOrientation, WriteMode};
use crate::plugin::input::InputPluginError;
use geojson::{Feature, FeatureCollection, GeoJson};
use routee_compass_core::util::geo::{geo_io_utils, PolygonalRTree};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "format")]
pub enum InjectPluginConfig {
    SpatialKeyValue(SpatialInjectPlugin),
    KeyValue(BasicInjectPlugin),
}

/// injects a value in each query at some key
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BasicInjectPlugin {
    pub key: String,
    pub value: Value,
    pub write_mode: WriteMode,
}
/// injects a value in each query at some key by first intersecting
/// the query map location with some geospatial dataset in order to
/// provide spatially-varied injection.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SpatialInjectPlugin {
    pub spatial_input_file: String,
    pub source_key: Option<String>,
    pub key: String,
    pub write_mode: WriteMode,
    pub orientation: CoordinateOrientation,
    pub default: Option<Value>,
}

impl InjectPluginConfig {
    pub fn build(&self) -> Result<InjectInputPlugin, InputPluginError> {
        match self {
            InjectPluginConfig::KeyValue(basic) => Ok(InjectInputPlugin::Basic {
                key: basic.key.clone(),
                value: basic.value.clone(),
                write_mode: basic.write_mode.clone(),
            }),
            InjectPluginConfig::SpatialKeyValue(spatial) => {
                let contents =
                    std::fs::read_to_string(&spatial.spatial_input_file).map_err(|e| {
                        InputPluginError::BuildFailed(format!(
                            "file read failed for file '{}': {}",
                            spatial.spatial_input_file, e
                        ))
                    })?;
                let geojson = contents.parse::<GeoJson>().map_err(|e| {
                    InputPluginError::BuildFailed(format!(
                        "unable to parse file '{}' as GeoJSON: {}",
                        spatial.spatial_input_file, e
                    ))
                })?;
                let fc = FeatureCollection::try_from(geojson).map_err(|e| {
                    InputPluginError::BuildFailed(format!(
                        "failed to unpack GeoJSON in file '{}' as a FeatureCollection: {}",
                        spatial.spatial_input_file, e
                    ))
                })?;
                let geometries_with_properties = fc
                    .features
                    .into_iter()
                    .map(|row| {
                        match &row.geometry {
                            None => Err(InputPluginError::BuildFailed(format!(
                                "row {} from GeoJSON in file '{}' has no geometry",
                                get_id(&row),
                                spatial.spatial_input_file
                            ))),
                            Some(g) => {
                                let geometry_f64: geo::Geometry<f64> = g.try_into().map_err(|e| {
                                    let msg = format!("row {} from GeoJSON in file '{}' has geometry that cannot be deserialized: {}", get_id(&row), spatial.spatial_input_file, e);
                                    InputPluginError::BuildFailed(msg)
                                })?;
                                // validate the geometry is polygonal
                                match geometry_f64 {
                                    geo::Geometry::Polygon(_) | geo::Geometry::MultiPolygon(_) => Ok(()),
                                    _ => Err(InputPluginError::BuildFailed(format!(
                                        "row {} from GeoJSON in file '{}' is not polygonal",
                                        get_id(&row), spatial.spatial_input_file
                                    ))),
                                }?;
                                let geometry = geo_io_utils::downsample_geometry(geometry_f64).map_err(|e| InputPluginError::BuildFailed(format!("row {} from GeoJSON in file '{}' has geometry that cannot be downsampled to f32: {}", get_id(&row), spatial.spatial_input_file, e)))?;

                                // 
                                let properties = match (&row.properties, spatial.source_key.clone()) {
                                    (None, _) => Err(InputPluginError::BuildFailed(format!("row {} from GeoJSON in file '{}' has no properties",
                                        get_id(&row), spatial.spatial_input_file))),
                                    (Some(props), None) => {
                                        Ok(serde_json::json!(props))
                                    },
                                    (Some(props), Some(props_key)) => {
                                        let value = props.get(&props_key).ok_or_else(|| InputPluginError::BuildFailed(format!("row {} from GeoJSON in file '{}' has no properties",
                                        get_id(&row), spatial.spatial_input_file)))?;
                                        Ok(value.clone())
                                    },
                                }?;

                                Ok((geometry, properties))
                            }
                        }
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                let spatial_index =
                    PolygonalRTree::new(geometries_with_properties).map_err(|e| {
                        InputPluginError::BuildFailed(format!(
                            "failed to build spatial index over GeoJSON input from file '{}': {}",
                            spatial.spatial_input_file, e
                        ))
                    })?;

                Ok(InjectInputPlugin::Spatial {
                    values: spatial_index,
                    key: spatial.key.clone(),
                    write_mode: spatial.write_mode.clone(),
                    orientation: spatial.orientation,
                    default: spatial.default.clone(),
                })
            }
        }
    }
}

fn get_id(row: &Feature) -> String {
    row.id
        .as_ref()
        .map(|i| format!("{:?}", i))
        .unwrap_or_default()
}
