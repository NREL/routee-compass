use super::{
    builders::{
        FrontierModelBuilder, InputPluginBuilder, OutputPluginBuilder, TraversalModelBuilder,
        TraversalModelService,
    },
    compass_configuration_error::CompassConfigurationError,
    compass_configuration_field::CompassConfigurationField,
    config_json_extension::ConfigJsonExtensions,
    frontier_model::{
        no_restriction_builder::NoRestrictionBuilder, road_class_builder::RoadClassBuilder,
    },
    traversal_model::{
        distance_builder::DistanceBuilder,
        speed_grade_energy_model_builder::SpeedGradeEnergyModelBuilder,
        speed_lookup_builder::SpeedLookupBuilder,
    },
};
use crate::plugin::{
    input::{
        default::{grid_search::builder::GridSearchBuilder, rtree::builder::VertexRTreeBuilder},
        input_plugin::InputPlugin,
    },
    output::{
        default::{
            edgeidlist::builder::EdgeIdListOutputPluginBuilder,
            summary::builder::SummaryOutputPluginBuilder,
            to_disk::builder::ToDiskOutputPluginBuilder,
            traversal::builder::TraversalPluginBuilder, uuid::builder::UUIDOutputPluginBuilder,
        },
        output_plugin::OutputPlugin,
    },
};
use routee_compass_core::model::frontier::frontier_model::FrontierModel;
use std::{collections::HashMap, sync::Arc};

/// Upstream component factory of [`crate::app::compass::compass_app::CompassApp`]
/// that builds components when constructing a CompassApp instance.
///
/// A [`CompassAppBuilder`] instance is typically created via the [`CompassAppBuilder::default']
/// method, which provides builders for commonly-used components.
/// Alternatively, there is a [`CompassAppBuilder::new'] method to build an empty instance.
/// Custom builder types can be added to an instance of CompassAppBuilder and
/// then loaded into a CompassApp when the configuration TOML input provides a `type` argument
/// signaling the key associated with the builder below.
///
/// Builders (values in the hash maps) are simple structs that have empty constructors and
/// no fields, so any number of these may be present without resulting in any loading.
/// It is only once these are referenced during CompassApp construction that files and models
/// will be loaded and CPU/RAM impacted.
///
/// # Arguments
///
/// * `tm_builders` - a mapping of TraversalModel `type` names to builders
/// * `frontier_builders` - a mapping of FrontierModel `type` names to builders
/// * `input_plugin_builders` - a mapping of InputPlugin `type` names to builders
/// * `output_plugin_builders` - a mapping of OutputPlugin `type` names to builders
///
pub struct CompassAppBuilder {
    pub tm_builders: HashMap<String, Box<dyn TraversalModelBuilder>>,
    pub frontier_builders: HashMap<String, Box<dyn FrontierModelBuilder>>,
    pub input_plugin_builders: HashMap<String, Box<dyn InputPluginBuilder>>,
    pub output_plugin_builders: HashMap<String, Box<dyn OutputPluginBuilder>>,
}

impl CompassAppBuilder {
    /// Build an empty [`CompassAppBuilder`] instance.
    /// If no additional builders are added, this will be unable to create
    /// components for a [`crate::app::compass::compass_app::CompassApp`],
    /// so this method is only useful is seeking a blank page for customizing.
    /// the [`CompassAppBuilder::default`] method provides the default builder set and is a better
    /// starting point.
    ///
    /// # Returns
    ///
    /// * an instance of a CompassAppBuilder that can be used to build a CompassApp
    pub fn new() -> CompassAppBuilder {
        CompassAppBuilder {
            tm_builders: HashMap::new(),
            frontier_builders: HashMap::new(),
            input_plugin_builders: HashMap::new(),
            output_plugin_builders: HashMap::new(),
        }
    }

    /// Builds the default builder.
    /// All components present in the routee-compass library are injected here
    /// into a builder instance with their expected `type` keys.
    ///
    /// # Returns
    ///
    /// * an instance of a CompassAppBuilder that can be used to build a CompassApp
    pub fn default() -> CompassAppBuilder {
        // Traversal model builders
        let dist: Box<dyn TraversalModelBuilder> = Box::new(DistanceBuilder {});
        let velo: Box<dyn TraversalModelBuilder> = Box::new(SpeedLookupBuilder {});
        let smartcore: Box<dyn TraversalModelBuilder> = Box::new(SpeedGradeEnergyModelBuilder {});
        let tm_builders: HashMap<String, Box<dyn TraversalModelBuilder>> = HashMap::from([
            (String::from("distance"), dist),
            (String::from("speed_table"), velo),
            (String::from("speed_grade_energy_model"), smartcore),
        ]);

        // Frontier model builders
        let no_restriction: Box<dyn FrontierModelBuilder> = Box::new(NoRestrictionBuilder {});
        let road_class: Box<dyn FrontierModelBuilder> = Box::new(RoadClassBuilder {});
        let frontier_builders: HashMap<String, Box<dyn FrontierModelBuilder>> = HashMap::from([
            (String::from("no_restriction"), no_restriction),
            (String::from("road_class"), road_class),
        ]);

        // Input plugin builders
        let vertex_tree: Box<dyn InputPluginBuilder> = Box::new(VertexRTreeBuilder {});
        let grid_search: Box<dyn InputPluginBuilder> = Box::new(GridSearchBuilder {});
        let input_plugin_builders = HashMap::from([
            (String::from("grid_search"), grid_search),
            (String::from("vertex_rtree"), vertex_tree),
        ]);

        // Output plugin builders
        let traversal: Box<dyn OutputPluginBuilder> = Box::new(TraversalPluginBuilder {});
        let summary: Box<dyn OutputPluginBuilder> = Box::new(SummaryOutputPluginBuilder {});
        let uuid: Box<dyn OutputPluginBuilder> = Box::new(UUIDOutputPluginBuilder {});
        let edge_id_list: Box<dyn OutputPluginBuilder> = Box::new(EdgeIdListOutputPluginBuilder {});
        let to_disk: Box<dyn OutputPluginBuilder> = Box::new(ToDiskOutputPluginBuilder {});
        let output_plugin_builders = HashMap::from([
            (String::from("traversal"), traversal),
            (String::from("summary"), summary),
            (String::from("uuid"), uuid),
            (String::from("edge_id_list"), edge_id_list),
            (String::from("to_disk"), to_disk),
        ]);

        CompassAppBuilder {
            tm_builders,
            frontier_builders,
            input_plugin_builders,
            output_plugin_builders,
        }
    }

    /// builds a traversal model with the specified type name with the provided
    /// traversal model configuration JSON
    pub fn build_traversal_model_service(
        &self,
        config: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModelService>, CompassConfigurationError> {
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
        config: &serde_json::Value,
    ) -> Result<Vec<Box<dyn InputPlugin>>, CompassConfigurationError> {
        let input_plugins = config.get_config_array(
            CompassConfigurationField::InputPlugins.to_string(),
            CompassConfigurationField::Plugins.to_string(),
        )?;

        let mut plugins: Vec<Box<dyn InputPlugin>> = Vec::new();
        for plugin_json in input_plugins.into_iter() {
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
            let input_plugin = builder.build(&plugin_json)?;
            plugins.push(input_plugin);
        }
        return Ok(plugins);
    }

    pub fn build_output_plugins(
        &self,
        config: &serde_json::Value,
    ) -> Result<Vec<Box<dyn OutputPlugin>>, CompassConfigurationError> {
        let output_plugins = config.get_config_array(
            CompassConfigurationField::OutputPlugins.to_string(),
            CompassConfigurationField::Plugins.to_string(),
        )?;

        let mut plugins: Vec<Box<dyn OutputPlugin>> = Vec::new();
        for plugin_json in output_plugins.into_iter() {
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
            let output_plugin = builder.build(&plugin_json)?;
            plugins.push(output_plugin);
        }
        return Ok(plugins);
    }
}
