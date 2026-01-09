use crate::model::fieldname;

use super::{
    interpolation::InterpolationModel, model_type::ModelType, prediction_model_ops,
    smartcore::SmartcoreModel, PredictionModel, PredictionModelConfig,
};
use routee_compass_core::model::{
    state::{InputFeature, StateModel, StateVariable},
    traversal::TraversalModelError,
    unit::{EnergyRateUnit, EnergyUnit},
};
use std::sync::{Arc, Mutex};
use uom::si::f64::{Energy, Mass};

/// A struct to hold the prediction model and associated metadata
pub struct PredictionModelRecord {
    pub name: String,
    pub prediction_model: Arc<dyn PredictionModel>,
    pub model_type: ModelType,
    pub input_features: Vec<InputFeature>,
    pub energy_rate_unit: EnergyRateUnit,
    pub mass_estimate: Mass,
    pub a_star_heuristic_energy_rate: f64,
    pub real_world_energy_adjustment: f64,
    // Cached indices for performance (Mutex for interior mutability)
    edge_distance_idx: Mutex<Option<usize>>,
    input_feature_indices: Mutex<Option<Vec<usize>>>,
}

impl TryFrom<&PredictionModelConfig> for PredictionModelRecord {
    type Error = TraversalModelError;

    fn try_from(config: &PredictionModelConfig) -> Result<Self, Self::Error> {
        if config.input_features.is_empty() {
            return Err(TraversalModelError::BuildError(format!(
                "You must supply at least one input feature for vehicle model {}",
                config.name
            )));
        }
        let prediction_model: Arc<dyn PredictionModel> = match &config.model_type {
            ModelType::Smartcore => {
                let model = SmartcoreModel::new(&config.model_input_file, config.energy_rate_unit)?;
                Arc::new(model)
            }
            ModelType::Interpolate {
                underlying_model_type: underlying_model,
                feature_bounds,
            } => {
                let model = InterpolationModel::new(
                    &config.model_input_file,
                    *underlying_model.clone(),
                    config.input_features.clone(),
                    feature_bounds.clone(),
                    config.energy_rate_unit,
                )?;
                Arc::new(model)
            }
        };

        let a_star_heuristic_energy_rate = match config.a_star_heuristic_energy_rate {
            None => prediction_model_ops::find_min_energy_rate(
                &prediction_model,
                &config.input_features,
                &config.energy_rate_unit,
            )?,
            Some(rate) => rate,
        };

        let real_world_energy_adjustment = config.real_world_energy_adjustment.unwrap_or(1.0);

        let mass_estimate = Mass::new::<uom::si::mass::pound>(config.mass_estimate_lbs);

        Ok(PredictionModelRecord {
            name: config.name.clone(),
            prediction_model,
            model_type: config.model_type.clone(),
            input_features: config.input_features.clone(),
            energy_rate_unit: config.energy_rate_unit,
            mass_estimate,
            a_star_heuristic_energy_rate,
            real_world_energy_adjustment,
            edge_distance_idx: Mutex::new(None),
            input_feature_indices: Mutex::new(None),
        })
    }
}

impl PredictionModelRecord {
    pub fn predict(
        &self,
        state: &mut [StateVariable],
        state_model: &StateModel,
    ) -> Result<Energy, TraversalModelError> {
        // Resolve edge_distance_idx if not cached
        let edge_distance_idx = {
            let mut cached = self.edge_distance_idx.lock().unwrap();
            if let Some(idx) = *cached {
                idx
            } else {
                let idx = state_model
                    .get_index(fieldname::EDGE_DISTANCE)
                    .map_err(|e| {
                        TraversalModelError::TraversalModelFailure(format!(
                            "Failed to find EDGE_DISTANCE index: {}",
                            e
                        ))
                    })?;
                *cached = Some(idx);
                idx
            }
        };

        // Resolve input feature indices if not cached
        let feature_indices = {
            let mut cached = self.input_feature_indices.lock().unwrap();
            if let Some(ref indices) = *cached {
                indices.clone()
            } else {
                let mut indices = Vec::with_capacity(self.input_features.len());
                for input_feature in &self.input_features {
                    let name = match input_feature {
                        InputFeature::Speed { name, .. } => name,
                        InputFeature::Ratio { name, .. } => name,
                        InputFeature::Temperature { name, .. } => name,
                        InputFeature::Custom { name, .. } => name,
                        _ => {
                            return Err(TraversalModelError::TraversalModelFailure(format!(
                                "got an unexpected input feature in the prediction model {input_feature}"
                            )));
                        }
                    };
                    let idx = state_model.get_index(name).map_err(|e| {
                        TraversalModelError::TraversalModelFailure(format!(
                            "Failed to find index for input feature {}: {}",
                            name, e
                        ))
                    })?;
                    indices.push(idx);
                }
                *cached = Some(indices.clone());
                indices
            }
        };

        let distance = state_model.get_distance_by_index(state, edge_distance_idx)?;
        let mut feature_vector: Vec<f64> = Vec::new();
        for (i, input_feature) in self.input_features.iter().enumerate() {
            let idx = feature_indices[i];
            let state_variable_f64: f64 = match input_feature {
                InputFeature::Speed { name: _, unit } => {
                    let speed = state_model.get_speed_by_index(state, idx)?;
                    match unit {
                        None => {
                            return Err(TraversalModelError::TraversalModelFailure(format!(
                                "Unit must be set for speed input feature {input_feature} but got None"
                            )));
                        }
                        Some(u) => u.from_uom(speed),
                    }
                }
                InputFeature::Ratio { name: _, unit } => {
                    let grade = state_model.get_ratio_by_index(state, idx)?;
                    match unit {
                        None => {
                            return Err(TraversalModelError::TraversalModelFailure(format!(
                                "Unit must be set for grade input feature {input_feature} but got None"
                            )));
                        }
                        Some(u) => u.from_uom(grade),
                    }
                }
                InputFeature::Temperature { name: _, unit } => {
                    let temperature = state_model.get_temperature_by_index(state, idx)?;
                    match unit {
                        None => {
                            return Err(TraversalModelError::TraversalModelFailure(format!(
                                "Unit must be set for temperature input feature {input_feature} but got None"
                            )));
                        }
                        Some(u) => u.from_uom(temperature),
                    }
                }
                InputFeature::Custom { name: _, unit: _ } => {
                    state_model.get_custom_f64_by_index(state, idx)?
                }
                _ => {
                    return Err(TraversalModelError::TraversalModelFailure(format!(
                        "got an unexpected input feature in the smartcore model prediction {input_feature}"
                    )))
                }
            };
            feature_vector.push(state_variable_f64);
        }

        let (energy_rate, energy_rate_unit) = self.prediction_model.predict(&feature_vector)?;

        let energy_rate_real_world = energy_rate * self.real_world_energy_adjustment;

        // TODO: This should be updated once we have EnergyRate as a UOM quantity
        let energy = match energy_rate_unit {
            EnergyRateUnit::GGPM => {
                let distance_miles = distance.get::<uom::si::length::mile>();
                let energy_f64 = energy_rate_real_world * distance_miles;
                EnergyUnit::GallonsGasolineEquivalent.to_uom(energy_f64)
            }
            EnergyRateUnit::GDPM => {
                let distance_miles = distance.get::<uom::si::length::mile>();
                let energy_f64 = energy_rate_real_world * distance_miles;
                EnergyUnit::GallonsDieselEquivalent.to_uom(energy_f64)
            }
            EnergyRateUnit::KWHPKM => {
                let distance_kilometers = distance.get::<uom::si::length::kilometer>();
                let energy_f64 = energy_rate_real_world * distance_kilometers;
                EnergyUnit::KilowattHours.to_uom(energy_f64)
            }
            EnergyRateUnit::KWHPM => {
                let distance_miles = distance.get::<uom::si::length::mile>();
                let energy_f64 = energy_rate_real_world * distance_miles;
                EnergyUnit::KilowattHours.to_uom(energy_f64)
            }
        };

        Ok(energy)
    }
}
