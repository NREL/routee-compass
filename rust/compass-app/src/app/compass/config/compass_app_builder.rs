use crate::{
    app::compass::compass_configuration_field::CompassConfigurationField,
    plugin::{
        input::{default::rtree::builder::VertexRTreeBuilder, input_plugin::InputPlugin},
        output::{
            default::{
                geometry::builder::GeometryPluginBuilder,
                summary::builder::SummaryOutputPluginBuilder,
                uuid::builder::UUIDOutputPluginBuilder,
            },
            output_plugin::OutputPlugin,
        },
    },
};

use super::{
    builders::{
        FrontierModelBuilder, InputPluginBuilder, OutputPluginBuilder, TraversalModelBuilder,
    },
    compass_configuration_error::CompassConfigurationError,
    frontier_model::{
        no_restriction_builder::NoRestrictionBuilder, road_class_builder::RoadClassBuilder,
    },
    traversal_model::{
        distance_builder::DistanceBuilder, energy_model_builder::EnergyModelBuilder,
        velocity_lookup_builder::VelocityLookupBuilder,
    },
};
use compass_core::model::{
    frontier::frontier_model::FrontierModel, traversal::traversal_model::TraversalModel,
};
use std::collections::HashMap;

pub struct CompassAppBuilder {
    pub tm_builders: HashMap<String, Box<dyn TraversalModelBuilder>>,
    pub frontier_builders: HashMap<String, Box<dyn FrontierModelBuilder>>,
    pub input_plugin_builders: HashMap<String, Box<dyn InputPluginBuilder>>,
    pub output_plugin_builders: HashMap<String, Box<dyn OutputPluginBuilder>>,
}

impl CompassAppBuilder {
    /// builds a traversal model with the specified type name with the provided
    /// traversal model configuration JSON
    pub fn build_traversal_model(
        &self,
        config: &serde_json::Value,
    ) -> Result<Box<dyn TraversalModel>, CompassConfigurationError> {
        let tm_type_obj =
            config
                .get("type")
                .ok_or(CompassConfigurationError::ExpectedFieldForComponent(
                    CompassConfigurationField::Traversal.to_string(),
                    String::from("type"),
                ))?;
        let tm_type: String = tm_type_obj
            .as_str()
            .ok_or(CompassConfigurationError::ExpectedFieldWithType(
                String::from("type"),
                String::from("String"),
            ))?
            .into();
        self.tm_builders
            .get(&tm_type)
            .ok_or(CompassConfigurationError::UnknownModelNameForComponent(
                tm_type.clone(),
                String::from("traversal"),
            ))
            .and_then(|b| b.build(config))
    }

    /// builds a frontier model with the specified type name with the provided
    /// frontier model configuration JSON
    pub fn build_frontier_model(
        &self,
        config: serde_json::Value,
    ) -> Result<Box<dyn FrontierModel>, CompassConfigurationError> {
        let fm_type_obj =
            config
                .get("type")
                .ok_or(CompassConfigurationError::ExpectedFieldForComponent(
                    CompassConfigurationField::Frontier.to_string(),
                    String::from("type"),
                ))?;
        let fm_type: String = fm_type_obj
            .as_str()
            .ok_or(CompassConfigurationError::ExpectedFieldWithType(
                String::from("type"),
                String::from("String"),
            ))?
            .into();
        self.frontier_builders
            .get(&fm_type)
            .ok_or(CompassConfigurationError::UnknownModelNameForComponent(
                fm_type.clone(),
                String::from("frontier"),
            ))
            .and_then(|b| b.build(&config))
    }

    pub fn build_input_plugins(
        &self,
        config: serde_json::Value,
    ) -> Result<Vec<Box<dyn InputPlugin>>, CompassConfigurationError> {
        let input_plugins_obj = config.get("input_plugins").ok_or(
            CompassConfigurationError::ExpectedFieldForComponent(
                CompassConfigurationField::InputPlugins.to_string(),
                String::from("input_plugins"),
            ),
        )?;
        let input_plugins = input_plugins_obj.as_array().ok_or(
            CompassConfigurationError::ExpectedFieldWithType(
                String::from("input_plugins"),
                String::from("Array"),
            ),
        )?;
        let mut plugins: Vec<Box<dyn InputPlugin>> = Vec::new();
        for plugin_json in input_plugins {
            let plugin_type_obj =
                plugin_json
                    .as_object()
                    .ok_or(CompassConfigurationError::ExpectedFieldWithType(
                        String::from("type"),
                        String::from("Json Object"),
                    ))?;
            let plugin_type: String = plugin_type_obj
                .get("type")
                .ok_or(CompassConfigurationError::ExpectedFieldForComponent(
                    CompassConfigurationField::InputPlugins.to_string(),
                    String::from("type"),
                ))?
                .as_str()
                .ok_or(CompassConfigurationError::ExpectedFieldWithType(
                    String::from("type"),
                    String::from("String"),
                ))?
                .into();
            let builder = self.input_plugin_builders.get(&plugin_type).ok_or(
                CompassConfigurationError::UnknownModelNameForComponent(
                    plugin_type.clone(),
                    String::from("Input Plugin"),
                ),
            )?;
            let input_plugin = builder.build(plugin_json)?;
            plugins.push(input_plugin);
        }
        return Ok(plugins);
    }

    pub fn build_output_plugins(
        &self,
        config: serde_json::Value,
    ) -> Result<Vec<Box<dyn OutputPlugin>>, CompassConfigurationError> {
        let output_plugins_obj = config.get("output_plugins").ok_or(
            CompassConfigurationError::ExpectedFieldForComponent(
                CompassConfigurationField::OutputPlugins.to_string(),
                String::from("output_plugins"),
            ),
        )?;
        let output_plugins = output_plugins_obj.as_array().ok_or(
            CompassConfigurationError::ExpectedFieldWithType(
                String::from("output_plugins"),
                String::from("Array"),
            ),
        )?;
        let mut plugins: Vec<Box<dyn OutputPlugin>> = Vec::new();
        for plugin_json in output_plugins {
            let plugin_json_obj =
                plugin_json
                    .as_object()
                    .ok_or(CompassConfigurationError::ExpectedFieldWithType(
                        String::from("output_plugins"),
                        String::from("Json Object"),
                    ))?;
            let plugin_type: String = plugin_json_obj
                .get("type")
                .ok_or(CompassConfigurationError::ExpectedFieldForComponent(
                    CompassConfigurationField::OutputPlugins.to_string(),
                    String::from("type"),
                ))?
                .as_str()
                .ok_or(CompassConfigurationError::ExpectedFieldWithType(
                    String::from("type"),
                    String::from("String"),
                ))?
                .into();
            let builder = self.output_plugin_builders.get(&plugin_type).ok_or(
                CompassConfigurationError::UnknownModelNameForComponent(
                    plugin_type.clone(),
                    String::from("Output Plugin"),
                ),
            )?;
            let output_plugin = builder.build(plugin_json)?;
            plugins.push(output_plugin);
        }
        return Ok(plugins);
    }

    /// builds the default builder which includes all defined components
    pub fn default() -> CompassAppBuilder {
        // Traversal model builders
        let dist: Box<dyn TraversalModelBuilder> = Box::new(DistanceBuilder {});
        let velo: Box<dyn TraversalModelBuilder> = Box::new(VelocityLookupBuilder {});
        let ener: Box<dyn TraversalModelBuilder> = Box::new(EnergyModelBuilder {});
        let tms: HashMap<String, Box<dyn TraversalModelBuilder>> = HashMap::from([
            (String::from("distance"), dist),
            (String::from("velocity_table"), velo),
            (String::from("energy"), ener),
        ]);

        // Frontier model builders
        let no_restriction: Box<dyn FrontierModelBuilder> = Box::new(NoRestrictionBuilder {});
        let road_class: Box<dyn FrontierModelBuilder> = Box::new(RoadClassBuilder {});
        let fms: HashMap<String, Box<dyn FrontierModelBuilder>> = HashMap::from([
            (String::from("no_restriction"), no_restriction),
            (String::from("road_class"), road_class),
        ]);

        // Input plugin builders
        let vertex_tree: Box<dyn InputPluginBuilder> = Box::new(VertexRTreeBuilder {});
        let input_builders = HashMap::from([(String::from("vertex_rtree"), vertex_tree)]);

        // Output plugin builders
        let geom: Box<dyn OutputPluginBuilder> = Box::new(GeometryPluginBuilder {});
        let summary: Box<dyn OutputPluginBuilder> = Box::new(SummaryOutputPluginBuilder {});
        let uuid: Box<dyn OutputPluginBuilder> = Box::new(UUIDOutputPluginBuilder {});
        let output_builders = HashMap::from([
            (String::from("geometry"), geom),
            (String::from("summary"), summary),
            (String::from("uuid"), uuid),
        ]);

        CompassAppBuilder {
            tm_builders: tms,
            frontier_builders: fms,
            input_plugin_builders: input_builders,
            output_plugin_builders: output_builders,
        }
    }
}
