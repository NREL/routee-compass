use crate::app::compass::compass_configuration_field::CompassConfigurationField;

use super::{
    builders::{FrontierModelBuilder, TraversalModelBuilder},
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
    frontier::{default::no_restriction::NoRestriction, frontier_model::FrontierModel},
    traversal::traversal_model::TraversalModel,
};
use std::collections::HashMap;

pub struct CompassAppBuilder {
    pub tm_builders: HashMap<String, Box<dyn TraversalModelBuilder>>,
    pub frontier_builders: HashMap<String, Box<dyn FrontierModelBuilder>>,
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

        return CompassAppBuilder {
            tm_builders: tms,
            frontier_builders: fms,
        };
    }
}
