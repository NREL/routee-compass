use crate::plugin::{
    input::{
        default::{
            debug::DebugInputPluginBuilder, grid_search::GridSearchBuilder,
            inject::InjectPluginBuilder, load_balancer::LoadBalancerBuilder,
        },
        InputPlugin, InputPluginBuilder,
    },
    output::{
        default::{
            summary::SummaryOutputPluginBuilder, traversal::TraversalPluginBuilder,
            uuid::UUIDOutputPluginBuilder,
        },
        OutputPlugin, OutputPluginBuilder,
    },
};
use itertools::Itertools;
use routee_compass_core::model::{
    access::{
        default::{
            combined_access_model_builder::CombinedAccessModelBuilder,
            turn_delays::TurnDelayAccessModelBuilder, NoAccessModel,
        },
        AccessModelBuilder, AccessModelService,
    },
    frontier::{
        default::{
            combined::combined_builder::CombinedFrontierModelBuilder,
            no_restriction_builder::NoRestrictionBuilder,
            road_class::road_class_builder::RoadClassBuilder,
            turn_restrictions::turn_restriction_builder::TurnRestrictionBuilder,
            vehicle_restrictions::VehicleRestrictionBuilder,
        },
        FrontierModelBuilder, FrontierModelService,
    },
    label::{
        default::vertex_label_model::VertexLabelModelBuilder,
        label_model_builder::LabelModelBuilder, label_model_service::LabelModelService,
    },
    traversal::{
        default::{
            combined::CombinedTraversalBuilder, custom::CustomTraversalBuilder,
            elevation::ElevationTraversalBuilder, grade::GradeTraversalBuilder,
            time::TimeTraversalBuilder,
        },
        TraversalModelBuilder, TraversalModelService,
    },
};
use routee_compass_core::{
    config::{CompassConfigurationError, CompassConfigurationField, ConfigJsonExtensions},
    model::traversal::default::{distance::DistanceTraversalBuilder, speed::SpeedTraversalBuilder},
};
use routee_compass_powertrain::model::{
    charging::{
        battery_frontier_builder::BatteryFrontierBuilder,
        simple_charging_builder::SimpleChargingBuilder, soc_label_builder::SOCLabelModelBuilder,
    },
    EnergyModelBuilder,
};
use std::{collections::HashMap, rc::Rc, sync::Arc};

use super::CompassComponentError;

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
    pub access_model_builders: HashMap<String, Rc<dyn AccessModelBuilder>>,
    pub frontier_model_builders: HashMap<String, Rc<dyn FrontierModelBuilder>>,
    pub label_model_builders: HashMap<String, Rc<dyn LabelModelBuilder>>,
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
            access_model_builders: HashMap::new(),
            frontier_model_builders: HashMap::new(),
            label_model_builders: HashMap::new(),
            input_plugin_builders: HashMap::new(),
            output_plugin_builders: HashMap::new(),
        }
    }

    pub fn add_traversal_model(&mut self, name: String, builder: Rc<dyn TraversalModelBuilder>) {
        let _ = self.traversal_model_builders.insert(name, builder);
    }

    pub fn add_access_model(&mut self, name: String, builder: Rc<dyn AccessModelBuilder>) {
        let _ = self.access_model_builders.insert(name, builder);
    }

    pub fn add_frontier_model(&mut self, name: String, builder: Rc<dyn FrontierModelBuilder>) {
        let _ = self.frontier_model_builders.insert(name, builder);
    }

    pub fn add_label_model(&mut self, name: String, builder: Rc<dyn LabelModelBuilder>) {
        let _ = self.label_model_builders.insert(name, builder);
    }

    pub fn add_input_plugin(&mut self, name: String, builder: Rc<dyn InputPluginBuilder>) {
        let _ = self.input_plugin_builders.insert(name, builder);
    }

    pub fn add_output_plugin(&mut self, name: String, builder: Rc<dyn OutputPluginBuilder>) {
        let _ = self.output_plugin_builders.insert(name, builder);
    }

    /// builds a traversal model with the specified type name with the provided
    /// traversal model configuration JSON
    pub fn build_traversal_model_service(
        &self,
        config: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModelService>, CompassConfigurationError> {
        // append the combined traversal model builder.
        let mut builders = self.traversal_model_builders.clone();
        builders.insert(
            String::from("combined"),
            Rc::new(CombinedTraversalBuilder::new(builders.clone())),
        );
        let tm_type = config.get_config_string(&"type", &"traversal")?;
        let result = builders
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

    pub fn build_access_model_service(
        &self,
        config: &serde_json::Value,
    ) -> Result<Arc<dyn AccessModelService>, CompassConfigurationError> {
        // append the combined access model builder.
        let mut builders = self.access_model_builders.clone();
        builders.insert(
            String::from("combined"),
            Rc::new(CombinedAccessModelBuilder::new(builders.clone())),
        );
        let tm_type = config.get_config_string(&"type", &"access")?;
        let result = builders
            .get(&tm_type)
            .ok_or_else(|| {
                CompassConfigurationError::UnknownModelNameForComponent(
                    tm_type.clone(),
                    String::from("access"),
                    self.access_model_builders.keys().join(", "),
                )
            })
            .and_then(|b| {
                b.build(config)
                    .map_err(CompassConfigurationError::AccessModelError)
            });
        result
    }

    /// builds a frontier model with the specified type name with the provided
    /// frontier model configuration JSON
    pub fn build_frontier_model_service(
        &self,
        config: &serde_json::Value,
    ) -> Result<Arc<dyn FrontierModelService>, CompassConfigurationError> {
        // append the combined frontier model builder.
        let mut builders = self.frontier_model_builders.clone();
        builders.insert(
            String::from("combined"),
            Rc::new(CombinedFrontierModelBuilder::new(builders.clone())),
        );
        let fm_type = config.get_config_string(&"type", &"frontier")?;
        builders
            .get(&fm_type)
            .ok_or_else(|| {
                CompassConfigurationError::UnknownModelNameForComponent(
                    fm_type.clone(),
                    String::from("frontier"),
                    self.frontier_model_builders.keys().join(", "),
                )
            })
            .and_then(|b| {
                b.build(config)
                    .map_err(CompassConfigurationError::FrontierModelError)
            })
    }

    pub fn build_label_model_service(
        &self,
        config: &serde_json::Value,
    ) -> Result<Arc<dyn LabelModelService>, CompassConfigurationError> {
        let lm_type = config.get_config_string(&"type", &"label")?;
        self.label_model_builders
            .get(&lm_type)
            .ok_or_else(|| {
                CompassConfigurationError::UnknownModelNameForComponent(
                    lm_type.clone(),
                    String::from("label"),
                    self.label_model_builders.keys().join(", "),
                )
            })
            .and_then(|b| {
                b.build(config)
                    .map_err(CompassConfigurationError::LabelModelError)
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
            let plugin_type = plugin_json.get_config_string(&"type", &"input_plugin")?;
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
    ) -> Result<Vec<Arc<dyn OutputPlugin>>, CompassComponentError> {
        let output_plugins = config.get_config_array(
            &CompassConfigurationField::OutputPlugins,
            &CompassConfigurationField::Plugins,
        )?;

        let mut plugins: Vec<Arc<dyn OutputPlugin>> = Vec::new();
        for plugin_json in output_plugins.into_iter() {
            let plugin_type = plugin_json.get_config_string(&"type", &"output_plugin")?;
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
    /// Builds the default builder.
    /// All components present in the routee-compass library are injected here
    /// into a builder instance with their expected `type` keys.
    ///
    /// The Combined variants which allow stringing together multiple models of
    /// the same type are automatically made available when constructing an instance of CompassApp.
    ///
    /// # Returns
    ///
    /// * an instance of a CompassAppBuilder that can be used to build a CompassApp
    fn default() -> Self {
        // Traversal model builders
        let distance: Rc<dyn TraversalModelBuilder> = Rc::new(DistanceTraversalBuilder {});
        let speed: Rc<dyn TraversalModelBuilder> = Rc::new(SpeedTraversalBuilder {});
        let time: Rc<dyn TraversalModelBuilder> = Rc::new(TimeTraversalBuilder {});
        let grade: Rc<dyn TraversalModelBuilder> = Rc::new(GradeTraversalBuilder {});
        let elevation: Rc<dyn TraversalModelBuilder> = Rc::new(ElevationTraversalBuilder {});
        let energy: Rc<dyn TraversalModelBuilder> = Rc::new(EnergyModelBuilder {});
        let simple_charging = Rc::new(SimpleChargingBuilder::default());
        let custom: Rc<dyn TraversalModelBuilder> = Rc::new(CustomTraversalBuilder {});
        let traversal_model_builders: HashMap<String, Rc<dyn TraversalModelBuilder>> =
            HashMap::from([
                (String::from("distance"), distance),
                (String::from("speed"), speed),
                (String::from("time"), time),
                (String::from("grade"), grade),
                (String::from("elevation"), elevation),
                (String::from("energy"), energy),
                (String::from("simple_charging"), simple_charging),
                (String::from("custom"), custom),
            ]);

        // Access model builders
        let no_access_model: Rc<dyn AccessModelBuilder> = Rc::new(NoAccessModel {});
        let turn_delay: Rc<dyn AccessModelBuilder> = Rc::new(TurnDelayAccessModelBuilder {});
        let access_model_builders: HashMap<String, Rc<dyn AccessModelBuilder>> = HashMap::from([
            (String::from("no_access_model"), no_access_model),
            (String::from("turn_delay"), turn_delay),
        ]);

        // Frontier model builders
        let no_restriction: Rc<dyn FrontierModelBuilder> = Rc::new(NoRestrictionBuilder {});
        let road_class: Rc<dyn FrontierModelBuilder> = Rc::new(RoadClassBuilder {});
        let turn_restriction: Rc<dyn FrontierModelBuilder> = Rc::new(TurnRestrictionBuilder {});
        let battery_frontier: Rc<dyn FrontierModelBuilder> =
            Rc::new(BatteryFrontierBuilder::default());
        let vehicle_restriction: Rc<dyn FrontierModelBuilder> =
            Rc::new(VehicleRestrictionBuilder {});
        let frontier_model_builders: HashMap<String, Rc<dyn FrontierModelBuilder>> =
            HashMap::from([
                (String::from("no_restriction"), no_restriction),
                (String::from("road_class"), road_class),
                (String::from("turn_restriction"), turn_restriction),
                (String::from("vehicle_restriction"), vehicle_restriction),
                (String::from("battery_frontier"), battery_frontier),
            ]);

        // Label model builders
        let vertex_label: Rc<dyn LabelModelBuilder> = Rc::new(VertexLabelModelBuilder);
        let soc_label = Rc::new(SOCLabelModelBuilder);
        let label_model_builders: HashMap<String, Rc<dyn LabelModelBuilder>> = HashMap::from([
            (String::from("vertex"), vertex_label),
            (String::from("soc"), soc_label),
        ]);

        // Input plugin builders
        let grid_search: Rc<dyn InputPluginBuilder> = Rc::new(GridSearchBuilder {});
        let load_balancer: Rc<dyn InputPluginBuilder> = Rc::new(LoadBalancerBuilder {});
        let inject: Rc<dyn InputPluginBuilder> = Rc::new(InjectPluginBuilder {});
        let debug: Rc<dyn InputPluginBuilder> = Rc::new(DebugInputPluginBuilder {});
        let input_plugin_builders = HashMap::from([
            (String::from("grid_search"), grid_search),
            (String::from("load_balancer"), load_balancer),
            (String::from("inject"), inject),
            (String::from("debug"), debug),
        ]);

        // Output plugin builders
        let traversal: Rc<dyn OutputPluginBuilder> = Rc::new(TraversalPluginBuilder {});
        let summary: Rc<dyn OutputPluginBuilder> = Rc::new(SummaryOutputPluginBuilder {});
        let uuid: Rc<dyn OutputPluginBuilder> = Rc::new(UUIDOutputPluginBuilder {});
        let output_plugin_builders = HashMap::from([
            (String::from("traversal"), traversal),
            (String::from("summary"), summary),
            (String::from("uuid"), uuid),
        ]);

        CompassAppBuilder {
            traversal_model_builders,
            access_model_builders,
            frontier_model_builders,
            label_model_builders,
            input_plugin_builders,
            output_plugin_builders,
        }
    }
}
