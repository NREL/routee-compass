use super::{CoordinateOrientation, WriteMode};
use crate::{
    app::search::SearchApp,
    plugin::input::{input_plugin::InputPlugin, InputJsonExtensions, InputPluginError},
};
use routee_compass_core::util::geo::PolygonalRTree;
use serde_json::Value;
use std::sync::Arc;

pub enum InjectInputPlugin {
    Basic {
        key: String,
        value: serde_json::Value,
        write_mode: WriteMode,
    },
    Spatial {
        values: PolygonalRTree<f32, Value>,
        key: String,
        write_mode: WriteMode,
        orientation: CoordinateOrientation,
        default: Option<Value>,
    },
}

impl InputPlugin for InjectInputPlugin {
    fn process(
        &self,
        input: &mut serde_json::Value,
        _search_app: Arc<SearchApp>,
    ) -> Result<(), InputPluginError> {
        process_inject(self, input)
    }
}

pub fn process_inject(
    plugin: &InjectInputPlugin,
    input: &mut serde_json::Value,
) -> Result<(), InputPluginError> {
    match plugin {
        InjectInputPlugin::Basic {
            key,
            value,
            write_mode,
        } => write_mode.write_to_query(input, key, value),
        InjectInputPlugin::Spatial {
            values,
            key,
            write_mode,
            orientation,
            default,
        } => {
            let coord = match orientation {
                CoordinateOrientation::Origin => input.get_origin_coordinate(),
                CoordinateOrientation::Destination => match input.get_destination_coordinate() {
                    Ok(Some(coord)) => Ok(coord),
                    Ok(None) => Err(InputPluginError::InputPluginFailed(String::from(
                        "destination-oriented spatial inject plugin but query has no destination",
                    ))),
                    Err(e) => Err(e),
                },
            }?;
            let point = geo::Geometry::Point(geo::Point(coord));
            let mut intersect_iter = values.intersection(&point).map_err(|e| {
                InputPluginError::InputPluginFailed(format!(
                    "failure while intersecting spatial inject data: {}",
                    e
                ))
            })?;
            match (intersect_iter.next(), default) {
                (None, None) => {
                    // nothing intersects + no default -> NOOP
                    Ok(())
                }
                (None, Some(default_value)) => {
                    // nothing intersects but we have a default
                    write_mode.write_to_query(input, key, default_value)
                }
                (Some(found), _) => {
                    // found an intersecting geometry with a value to assign
                    write_mode.write_to_query(input, key, &found.data)
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::{process_inject, InjectInputPlugin};
    use crate::plugin::input::default::inject::{
        inject_plugin_config::SpatialInjectPlugin, CoordinateOrientation, InjectPluginConfig,
        WriteMode,
    };
    use serde_json::{json, Value};
    use std::path::Path;

    #[test]
    fn test_basic() {
        let mut query = json!({});
        let key = String::from("key_on_query");
        let value = json![{"k": "v"}];
        let plugin = InjectInputPlugin::Basic {
            key: key.clone(),
            value: value.clone(),
            write_mode: WriteMode::Overwrite,
        };
        process_inject(&plugin, &mut query).expect("test failed");
        let result_value = query.get(&key).expect("test failed: key was not set");
        assert_eq!(
            result_value, &value,
            "test failed: value stored in GeoJSON with matching location does not match"
        )
    }

    #[test]
    fn test_spatial_contains() {
        let mut query = json!({
            "origin_x": -105.11011135094863,
            "origin_y": 39.83906153425838
        });
        let source_key = Some(String::from("key_on_geojson"));
        let key = String::from("key_on_query");
        let plugin = setup_spatial(&source_key, &key, &None);
        process_inject(&plugin, &mut query).expect("test failed");
        let value = query.get(&key).expect("test failed: key was not set");
        let value_number = value.as_i64().expect("test failed: value was not a number");
        assert_eq!(
            value_number, 5000,
            "test failed: value stored in GeoJSON with matching location was not injected"
        )
    }

    #[test]
    fn test_spatial_not_contains() {
        let mut query = json!({
            "origin_x": -105.07021837975549,
            "origin_y": 39.93602243844981
        });
        let source_key = Some(String::from("key_on_geojson"));
        let key = String::from("key_on_query");
        let default = String::from("found default");
        let plugin = setup_spatial(&source_key, &key, &Some(json![default.clone()]));
        process_inject(&plugin, &mut query).expect("test failed");
        let value = query.get(&key).expect("test failed: key was not set");
        let value_str = value.as_str().expect("test failed: value was not a str");
        assert_eq!(
            value_str, &default,
            "test failed: value stored in GeoJSON with location that does not match did not return default fill value"
        )
    }

    #[test]
    fn test_spatial_not_contains_no_default() {
        let mut query = json!({
            "origin_x": -105.07021837975549,
            "origin_y": 39.93602243844981
        });
        let expected = query.clone();
        let source_key = Some(String::from("key_on_geojson"));
        let key = String::from("key_on_query");
        let plugin = setup_spatial(&source_key, &key, &None);
        process_inject(&plugin, &mut query).expect("test failed");
        assert_eq!(
            query, expected,
            "test failed: process should be idempotent when the query is not contained and there is no default value"
        )
    }

    #[test]
    fn test_spatial_from_json() {
        let source_key = String::from("key_on_geojson");
        let key = String::from("key_on_query");
        let conf_str = format!(
            r#"
        {{
            "type": "spatial",
            "spatial_input_file": "{}",
            "source_key": "{}",
            "key": "{}",
            "write_mode": "overwrite",
            "orientation": "origin"
        }}
        "#,
            test_filepath(),
            &source_key,
            &key
        );
        let conf: InjectPluginConfig =
            serde_json::from_str(&conf_str).expect("failed to decode configuration");
        let plugin = conf.build().expect("failed to build plugin");
        let mut query = json!({
            "origin_x": -105.11011135094863,
            "origin_y": 39.83906153425838
        });

        process_inject(&plugin, &mut query).expect("failed to run plugin");
        let value = query.get(&key).expect("test failed: key was not set");
        let value_number = value.as_i64().expect("test failed: value was not a number");
        assert_eq!(
            value_number, 5000,
            "test failed: value stored in GeoJSON with matching location was not injected"
        )
    }

    fn test_filepath() -> String {
        let spatial_input_filepath = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("plugin")
            .join("input")
            .join("default")
            .join("inject")
            .join("test")
            .join("test.geojson");
        spatial_input_filepath.to_string_lossy().to_string()
    }

    fn setup_spatial(
        source_key: &Option<String>,
        key: &String,
        default: &Option<Value>,
    ) -> InjectInputPlugin {
        let spatial_input_file = test_filepath();
        let conf = InjectPluginConfig::SpatialKeyValue(SpatialInjectPlugin {
            spatial_input_file,
            source_key: source_key.clone(),
            key: key.clone(),
            write_mode: WriteMode::Overwrite,
            orientation: CoordinateOrientation::Origin,
            default: default.clone(),
        });

        conf.build().expect("test invariant failed")
    }
}
