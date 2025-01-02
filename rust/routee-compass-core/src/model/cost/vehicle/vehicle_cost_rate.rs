use crate::model::{
    state::StateVariable,
    unit::{AsF64, Cost},
};
use serde::{Deserialize, Serialize};
/// a mapping for how to transform vehicle state values into a Cost.
/// mappings can be a single instance of Raw, Factor, or Offset mapping.
///
/// when multiple mappings are specified they are applied sequentially (in user-defined order)
/// to the state value.
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum VehicleCostRate {
    /// no cost
    #[default]
    Zero,
    /// use a value directly as a cost
    Raw,
    /// multiply a value by a factor to become a cost
    Factor {
        factor: f64,
    },
    /// shift a value by some amount
    Offset {
        offset: f64,
    },
    Combined(Vec<VehicleCostRate>),
    // leaving room for extension if we need to do any fancier math, maybe not needed
    // Poly2 { x0: f64, x1: f64 },
    // Exp { base: f64, exp_coefficient: f64 },
}

impl VehicleCostRate {
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
    pub fn map_value(&self, state: StateVariable) -> Cost {
        match self {
            VehicleCostRate::Zero => Cost::ZERO,
            VehicleCostRate::Raw => Cost::new(state.0),
            VehicleCostRate::Factor { factor } => Cost::new(state.0 * factor),
            VehicleCostRate::Offset { offset } => Cost::new(state.0 + offset),
            VehicleCostRate::Combined(mappings) => {
                mappings.iter().fold(Cost::new(state.0), |acc, f| {
                    f.map_value(StateVariable(acc.as_f64()))
                })
            }
        }
    }
}
