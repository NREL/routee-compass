use crate::{
    model::{traversal::state::state_variable::StateVar, utility::cost::Cost},
    util::unit::as_f64::AsF64,
};

/// a mapping for how to transform vehicle state values into a Cost.
/// mappings can be a single instance of Raw, Factor, or Offset mapping.
///
/// when multiple mappings are specified they are applied sequentially (in user-defined order)
/// to the state value.
pub enum VehicleUtilityMapping {
    /// use a value directly as a cost
    Raw,
    /// multiply a value by a factor to become a cost
    Factor {
        factor: f64,
    },
    ///
    Offset {
        offset: f64,
    },
    Combined(Vec<VehicleUtilityMapping>),
    // leaving room for extension if we need to do any fancier math, maybe not needed
    // Poly2 { x0: f64, x1: f64 },
    // Exp { base: f64, exp_coefficient: f64 },
}

impl VehicleUtilityMapping {
    /// maps a state variable to a Cost value based on a user-configured mapping.
    ///
    /// # Arguments
    ///
    /// * `state` - the state variable to map to a Cost value
    ///
    /// # Result
    ///
    /// the Cost value for that state, a real number that is aggregated with
    /// other Cost values in a common unit space.
    pub fn map_value(&self, state: StateVar) -> Cost {
        match self {
            VehicleUtilityMapping::Raw => Cost::new(state.0),
            VehicleUtilityMapping::Factor { factor } => Cost::new(state.0 * factor),
            VehicleUtilityMapping::Offset { offset } => Cost::new(state.0 + offset),
            VehicleUtilityMapping::Combined(mappings) => {
                mappings.iter().fold(Cost::new(state.0), |acc, f| {
                    f.map_value(StateVar(acc.as_f64()))
                })
            }
        }
    }
}