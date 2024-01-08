use super::{
    builders::{InputPluginBuilder, OutputPluginBuilder},
    compass_configuration_error::CompassConfigurationError,
    compass_configuration_field::CompassConfigurationField,
    config_json_extension::ConfigJsonExtensions,
    frontier_model::{
        combined::combined_builder::CombinedBuilder, no_restriction_builder::NoRestrictionBuilder,
        road_class::road_class_builder::RoadClassBuilder,
        turn_restrictions::turn_restriction_builder::TurnRestrictionBuilder,
    },
    traversal_model::{
        distance_builder::DistanceBuilder, energy_model_builder::EnergyModelBuilder,
        speed_lookup_builder::SpeedLookupBuilder,
    },
};
use crate::plugin::{
    input::{
        default::{
            debug::debug_builder::DebugInputPluginBuilder,
            edge_rtree::edge_rtree_input_plugin_builder::EdgeRtreeInputPluginBuilder,
            grid_search::builder::GridSearchBuilder, inject::inject_builder::InjectPluginBuilder,
            load_balancer::builder::LoadBalancerBuilder, vertex_rtree::builder::VertexRTreeBuilder,
        },
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

use itertools::Itertools;
use routee_compass_core::model::{
    frontier::{
        frontier_model_builder::FrontierModelBuilder, frontier_model_service::FrontierModelService,
    },
    traversal::{
        traversal_model_builder::TraversalModelBuilder,
        traversal_model_service::TraversalModelService,
    },
};
use std::{collections::HashMap, rc::Rc, sync::Arc};

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
    pub traversal_model_builders: HashMap<String, Rc<dyn TraversalModelBuilder>>,
    pub frontier_builders: HashMap<String, Rc<dyn FrontierModelBuilder>>,
    pub input_plugin_builders: HashMap<String, Rc<dyn InputPluginBuilder>>,
    pub output_plugin_builders: HashMap<String, Rc<dyn OutputPluginBuilder>>,
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
            traversal_model_builders: HashMap::new(),
            frontier_builders: HashMap::new(),
            input_plugin_builders: HashMap::new(),
            output_plugin_builders: HashMap::new(),
        }
    }

    pub fn add_traversal_model(&mut self, name: String, builder: Rc<dyn TraversalModelBuilder>) {
        let _ = self.traversal_model_builders.insert(name, builder);
    }

    pub fn add_frontier_model(&mut self, name: String, builder: Rc<dyn FrontierModelBuilder>) {
        let _ = self.frontier_builders.insert(name, builder);
    }

    pub fn add_input_plugin(&mut self, name: String, builder: Rc<dyn InputPluginBuilder>) {
        let _ = self.input_plugin_builders.insert(name, builder);
    }

    pub fn add_output_plugin(&mut self, name: String, builder: Rc<dyn OutputPluginBuilder>) {
        let _ = self.output_plugin_builders.insert(name, builder);
    }

    /// Builds the default builder.
    /// All components present in the routee-compass library are injected here
    /// into a builder instance with their expected `type` keys.
    ///
    /// # Returns
    ///
    /// * an instance of a CompassAppBuilder that can be used to build a CompassApp
    fn default() -> CompassAppBuilder {
        // Traversal model builders
        let dist: Rc<dyn TraversalModelBuilder> = Rc::new(DistanceBuilder {});
        let velo: Rc<dyn TraversalModelBuilder> = Rc::new(SpeedLookupBuilder {});
        let energy_model: Rc<dyn TraversalModelBuilder> = Rc::new(EnergyModelBuilder {});
        let tm_builders: HashMap<String, Rc<dyn TraversalModelBuilder>> = HashMap::from([
            (String::from("distance"), dist),
            (String::from("speed_table"), velo),
            (String::from("energy_model"), energy_model),
        ]);

        // Frontier model builders
        let no_restriction: Rc<dyn FrontierModelBuilder> = Rc::new(NoRestrictionBuilder {});
        let road_class: Rc<dyn FrontierModelBuilder> = Rc::new(RoadClassBuilder {});
        let turn_restruction: Rc<dyn FrontierModelBuilder> = Rc::new(TurnRestrictionBuilder {});
        let base_frontier_builders: HashMap<String, Rc<dyn FrontierModelBuilder>> =
            HashMap::from([
                (String::from("no_restriction"), no_restriction),
                (String::from("road_class"), road_class),
                (String::from("turn_restriction"), turn_restruction),
            ]);
        let combined = Rc::new(CombinedBuilder {
            builders: base_frontier_builders.clone(),
        });
        let mut all_frontier_builders = base_frontier_builders.clone();
        all_frontier_builders.insert(String::from("combined"), combined);

        // Input plugin builders
        let grid_search: Rc<dyn InputPluginBuilder> = Rc::new(GridSearchBuilder {});
        let vertex_tree: Rc<dyn InputPluginBuilder> = Rc::new(VertexRTreeBuilder {});
        let edge_rtree: Rc<dyn InputPluginBuilder> = Rc::new(EdgeRtreeInputPluginBuilder {});
        let load_balancer: Rc<dyn InputPluginBuilder> = Rc::new(LoadBalancerBuilder {});
        let inject: Rc<dyn InputPluginBuilder> = Rc::new(InjectPluginBuilder {});
        let debug: Rc<dyn InputPluginBuilder> = Rc::new(DebugInputPluginBuilder {});
        let input_plugin_builders = HashMap::from([
            (String::from("grid_search"), grid_search),
            (String::from("vertex_rtree"), vertex_tree),
            (String::from("edge_rtree"), edge_rtree),
            (String::from("load_balancer"), load_balancer),
            (String::from("inject"), inject),
            (String::from("debug"), debug),
        ]);

        // Output plugin builders
        let traversal: Rc<dyn OutputPluginBuilder> = Rc::new(TraversalPluginBuilder {});
        let summary: Rc<dyn OutputPluginBuilder> = Rc::new(SummaryOutputPluginBuilder {});
        let uuid: Rc<dyn OutputPluginBuilder> = Rc::new(UUIDOutputPluginBuilder {});
        let edge_id_list: Rc<dyn OutputPluginBuilder> = Rc::new(EdgeIdListOutputPluginBuilder {});
        let to_disk: Rc<dyn OutputPluginBuilder> = Rc::new(ToDiskOutputPluginBuilder {});
        let output_plugin_builders = HashMap::from([
            (String::from("traversal"), traversal),
            (String::from("summary"), summary),
            (String::from("uuid"), uuid),
            (String::from("edge_id_list"), edge_id_list),
            (String::from("to_disk"), to_disk),
        ]);

        CompassAppBuilder {
            traversal_model_builders: tm_builders,
            frontier_builders: all_frontier_builders,
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
        let tm_type_obj = config.get("type").ok_or_else(|| {
            CompassConfigurationError::ExpectedFieldForComponent(
                CompassConfigurationField::Traversal.to_string(),
                String::from("type"),
            )
        })?;
        let tm_type: String = tm_type_obj
            .as_str()
            .ok_or_else(|| {
                CompassConfigurationError::ExpectedFieldWithType(
                    String::from("type"),
                    String::from("String"),
                )
            })?
            .into();
        let result = self
            .traversal_model_builders
            .get(&tm_type)
            .ok_or_else(|| {
                CompassConfigurationError::UnknownModelNameForComponent(
                    tm_type.clone(),
                    String::from("traversal"),
                    self.traversal_model_builders.keys().join(", "),
                )
            })
            .and_then(|b| {
                b.build(config)
                    .map_err(CompassConfigurationError::TraversalModelError)
            });
        result
    }

    /// builds a frontier model with the specified type name with the provided
    /// frontier model configuration JSON
    pub fn build_frontier_model_service(
        &self,
        config: &serde_json::Value,
    ) -> Result<Arc<dyn FrontierModelService>, CompassConfigurationError> {
        let fm_type_obj = config.get("type").ok_or_else(|| {
            CompassConfigurationError::ExpectedFieldForComponent(
                CompassConfigurationField::Frontier.to_string(),
                String::from("type"),
            )
        })?;
        let fm_type: String = fm_type_obj
            .as_str()
            .ok_or_else(|| {
                CompassConfigurationError::ExpectedFieldWithType(
                    String::from("type"),
                    String::from("String"),
                )
            })?
            .into();
        self.frontier_builders
            .get(&fm_type)
            .ok_or_else(|| {
                CompassConfigurationError::UnknownModelNameForComponent(
                    fm_type.clone(),
                    String::from("frontier"),
                    self.frontier_builders.keys().join(", "),
                )
            })
            .and_then(|b| {
                b.build(config)
                    .map_err(CompassConfigurationError::FrontierModelError)
            })
    }

    pub fn build_input_plugins(
        &self,
        config: &serde_json::Value,
    ) -> Result<Vec<Arc<dyn InputPlugin>>, CompassConfigurationError> {
        let input_plugins = config.get_config_array(
            &CompassConfigurationField::InputPlugins,
            &CompassConfigurationField::Plugins,
        )?;

        let mut plugins: Vec<Arc<dyn InputPlugin>> = Vec::new();
        for plugin_json in input_plugins.into_iter() {
            let plugin_type_obj = plugin_json.as_object().ok_or_else(|| {
                CompassConfigurationError::ExpectedFieldWithType(
                    String::from("type"),
                    String::from("Json Object"),
                )
            })?;
            let plugin_type: String = plugin_type_obj
                .get("type")
                .ok_or_else(|| {
                    CompassConfigurationError::ExpectedFieldForComponent(
                        CompassConfigurationField::InputPlugins.to_string(),
                        String::from("type"),
                    )
                })?
                .as_str()
                .ok_or_else(|| {
                    CompassConfigurationError::ExpectedFieldWithType(
                        String::from("type"),
                        String::from("String"),
                    )
                })?
                .into();
            let builder = self
                .input_plugin_builders
                .get(&plugin_type)
                .ok_or_else(|| {
                    CompassConfigurationError::UnknownModelNameForComponent(
                        plugin_type.clone(),
                        String::from("Input Plugin"),
                        self.input_plugin_builders.keys().join(", "),
                    )
                })?;
            let input_plugin = builder.build(&plugin_json)?;
            plugins.push(input_plugin);
        }
        Ok(plugins)
    }

    pub fn build_output_plugins(
        &self,
        config: &serde_json::Value,
    ) -> Result<Vec<Arc<dyn OutputPlugin>>, CompassConfigurationError> {
        let output_plugins = config.get_config_array(
            &CompassConfigurationField::OutputPlugins,
            &CompassConfigurationField::Plugins,
        )?;

        let mut plugins: Vec<Arc<dyn OutputPlugin>> = Vec::new();
        for plugin_json in output_plugins.into_iter() {
            let plugin_json_obj = plugin_json.as_object().ok_or_else(|| {
                CompassConfigurationError::ExpectedFieldWithType(
                    String::from("output_plugins"),
                    String::from("Json Object"),
                )
            })?;
            let plugin_type: String = plugin_json_obj
                .get("type")
                .ok_or_else(|| {
                    CompassConfigurationError::ExpectedFieldForComponent(
                        CompassConfigurationField::OutputPlugins.to_string(),
                        String::from("type"),
                    )
                })?
                .as_str()
                .ok_or_else(|| {
                    CompassConfigurationError::ExpectedFieldWithType(
                        String::from("type"),
                        String::from("String"),
                    )
                })?
                .into();
            let builder = self
                .output_plugin_builders
                .get(&plugin_type)
                .ok_or_else(|| {
                    CompassConfigurationError::UnknownModelNameForComponent(
                        plugin_type.clone(),
                        String::from("Output Plugin"),
                        self.output_plugin_builders.keys().join(", "),
                    )
                })?;
            let output_plugin = builder.build(&plugin_json)?;
            plugins.push(output_plugin);
        }
        Ok(plugins)
    }
}

impl Default for CompassAppBuilder {
    fn default() -> Self {
        CompassAppBuilder::default()
    }
}
