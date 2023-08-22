use super::{
    compass_configuration_error::CompassConfigurationError,
    traversal_model::{
        distance_builder::DistanceBuilder, energy_model_builder::EnergyModelBuilder,
        velocity_lookup_builder::VelocityLookupBuilder,
    },
    traversal_model_builder::TraversalModelBuilder,
};
use compass_core::model::traversal::traversal_model::TraversalModel;
use std::collections::HashMap;

pub struct CompassAppBuilder {
    pub tm_builders: HashMap<String, Box<dyn TraversalModelBuilder>>,
}

impl CompassAppBuilder {
    // pub fn add_tm_builder(&mut self, name: String, builder: Box<dyn TraversalModelBuilder>) {
    //     match self.tm_builders.insert(name.clone(), builder) {
    //         Some(_prev) => log::warn!("adding traversal model builder {} more than once", &name),
    //         None => (),
    //     }
    // }

    /// builds a traversal model with the specified type name with the provided
    /// traversal model configuration JSON
    pub fn build_traversal_model(
        &self,
        type_name: &String,
        config: &serde_json::Value,
    ) -> Result<Box<dyn TraversalModel>, CompassConfigurationError> {
        self.tm_builders
            .get(type_name)
            .ok_or(CompassConfigurationError::UnknownModelNameForComponent(
                type_name.clone(),
                String::from("traversal"),
            ))
            .and_then(|b| b.build(config))
    }

    pub fn default() -> CompassAppBuilder {
        let dist: Box<dyn TraversalModelBuilder> = Box::new(DistanceBuilder {});
        let velo: Box<dyn TraversalModelBuilder> = Box::new(VelocityLookupBuilder {});
        let ener: Box<dyn TraversalModelBuilder> = Box::new(EnergyModelBuilder {});
        let tms: HashMap<String, Box<dyn TraversalModelBuilder>> = HashMap::from([
            (String::from("distance"), dist),
            (String::from("velocity_table"), velo),
            (String::from("energy"), ener),
        ]);
        return CompassAppBuilder { tm_builders: tms };
    }
}
