use crate::model::fieldname;

use super::{
    energy_model_ops,
    prediction::{PredictionModelConfig, PredictionModelRecord},
};
use routee_compass_core::model::{
    network::{Edge, Vertex},
    state::{CustomFeatureFormat, InputFeature, OutputFeature, StateModel, StateVariable},
    traversal::{TraversalModel, TraversalModelError, TraversalModelService},
    unit::{Energy, EnergyUnit},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

#[derive(Clone)]
pub struct PhevEnergyModel {
    /// liquid fuel model used when battery is depleted
    pub charge_sustain_model: Arc<PredictionModelRecord>,
    /// electric fuel model used when battery is non-zero
    pub charge_depleting_model: Arc<PredictionModelRecord>,
    pub battery_capacity: (Energy, EnergyUnit),
    pub starting_soc: f64,
}

impl PhevEnergyModel {
    pub fn new(
        charge_sustain_model: Arc<PredictionModelRecord>,
        charge_depleting_model: Arc<PredictionModelRecord>,
        battery_capacity: (Energy, EnergyUnit),
        starting_battery_energy: (Energy, EnergyUnit),
    ) -> Result<Self, TraversalModelError> {
        let starting_energy = (&starting_battery_energy.0, &starting_battery_energy.1);
        let bat_cap_ref = (&battery_capacity.0, &battery_capacity.1);
        let starting_soc = energy_model_ops::soc_from_energy(starting_energy, bat_cap_ref)
            .map_err(TraversalModelError::BuildError)?;
        Ok(Self {
            charge_sustain_model,
            charge_depleting_model,
            battery_capacity,
            starting_soc,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PhevEnergyModelConfig {
    pub charge_sustaining: PredictionModelConfig,
    pub charge_depleting: PredictionModelConfig,
    pub battery_capacity: Energy,
    pub battery_capacity_unit: EnergyUnit,
}

impl TryFrom<&Value> for PhevEnergyModel {
    type Error = TraversalModelError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let config: PhevEnergyModelConfig = serde_json::from_value(value.clone()).map_err(|e| {
            TraversalModelError::BuildError(format!(
                "failure reading phev energy model configuration: {}",
                e
            ))
        })?;
        let charge_depleting_record = PredictionModelRecord::try_from(&config.charge_depleting)?;
        let charge_sustaining_record = PredictionModelRecord::try_from(&config.charge_sustaining)?;
        let bev = PhevEnergyModel::new(
            Arc::new(charge_sustaining_record),
            Arc::new(charge_depleting_record),
            (config.battery_capacity, config.battery_capacity_unit),
            (config.battery_capacity, config.battery_capacity_unit),
        )?;
        Ok(bev)
    }
}

impl TraversalModelService for PhevEnergyModel {
    fn build(
        &self,
        query: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModel>, TraversalModelError> {
        let (capacity, capacity_unit) = (self.battery_capacity.0, self.battery_capacity.1);
        match energy_model_ops::get_query_start_energy(query, &capacity)? {
            None => Ok(Arc::new(self.clone())),
            Some(starting_energy) => {
                let updated = Self::new(
                    self.charge_sustain_model.clone(),
                    self.charge_depleting_model.clone(),
                    (capacity, capacity_unit),
                    (starting_energy, capacity_unit),
                )?;
                Ok(Arc::new(updated))
            }
        }
    }
}

impl TraversalModel for PhevEnergyModel {
    fn input_features(&self) -> Vec<(String, InputFeature)> {
        vec![
            (
                String::from(fieldname::EDGE_DISTANCE),
                InputFeature::Distance(None),
            ),
            (
                String::from(fieldname::EDGE_SPEED),
                InputFeature::Speed(None),
            ),
            (
                String::from(fieldname::EDGE_GRADE),
                InputFeature::Grade(None),
            ),
        ]
    }

    fn output_features(&self) -> Vec<(String, OutputFeature)> {
        let liquid_energy_feature = OutputFeature::Energy {
            energy_unit: self
                .charge_sustain_model
                .energy_rate_unit
                .associated_energy_unit(),
            initial: Energy::ZERO,
        };
        let electric_energy_feature = OutputFeature::Energy {
            energy_unit: self
                .charge_depleting_model
                .energy_rate_unit
                .associated_energy_unit(),
            initial: Energy::ZERO,
        };
        vec![
            (
                String::from(fieldname::TRIP_ENERGY_LIQUID),
                liquid_energy_feature.clone(),
            ),
            (
                String::from(fieldname::EDGE_ENERGY_LIQUID),
                liquid_energy_feature.clone(),
            ),
            (
                String::from(fieldname::TRIP_ENERGY_ELECTRIC),
                electric_energy_feature.clone(),
            ),
            (
                String::from(fieldname::EDGE_ENERGY_ELECTRIC),
                electric_energy_feature.clone(),
            ),
            (
                String::from(fieldname::TRIP_SOC),
                OutputFeature::Custom {
                    r#type: String::from("soc"),
                    unit: String::from("Percent"),
                    format: CustomFeatureFormat::FloatingPoint {
                        initial: self.starting_soc.into(),
                    },
                },
            ),
        ]
    }

    fn traverse_edge(
        &self,
        _trajectory: (&Vertex, &Edge, &Vertex),
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        phev_traversal(
            state,
            state_model,
            self.charge_depleting_model.clone(),
            self.charge_sustain_model.clone(),
            (&self.battery_capacity.0, &self.battery_capacity.1),
        )
    }

    /// estimates energy use based only on using the liquid fuel model
    fn estimate_traversal(
        &self,
        _od: (&Vertex, &Vertex),
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        let distance = state_model.get_distance(state, fieldname::EDGE_DISTANCE, None)?;
        let speed = state_model.get_speed(state, fieldname::EDGE_SPEED, None)?;
        let grade = state_model.get_grade(state, fieldname::EDGE_GRADE, None)?;
        let (energy, energy_unit) = self.charge_sustain_model.predict(speed, grade, distance)?;
        state_model.set_energy(state, fieldname::EDGE_ENERGY_LIQUID, &energy, &energy_unit)?;
        state_model.add_energy(state, fieldname::TRIP_ENERGY_LIQUID, &energy, &energy_unit)?;
        Ok(())
    }
}

/// a PHEV traversal that switches between a charge depleting and charge sustaining model in order
/// to compute energy consumption. the mechanism checks for battery state of charge, and if it is
/// greater than 0, then the charge depleting model is used. otherwise, the charge sustaining model is used.
fn phev_traversal(
    state: &mut [StateVariable],
    state_model: &StateModel,
    charge_depleting_model: Arc<PredictionModelRecord>,
    charge_sustaining_model: Arc<PredictionModelRecord>,
    battery_capacity: (&Energy, &EnergyUnit),
) -> Result<(), TraversalModelError> {
    let distance = state_model.get_distance(state, fieldname::EDGE_DISTANCE, None)?;
    let speed = state_model.get_speed(state, fieldname::EDGE_SPEED, None)?;
    let grade = state_model.get_grade(state, fieldname::EDGE_GRADE, None)?;
    let start_soc = state_model.get_custom_f64(state, fieldname::TRIP_SOC)?;
    if start_soc > 0.0 {
        // use electric model
        let (energy, energy_unit) = charge_depleting_model.predict(speed, grade, distance)?;
        state_model.set_energy(
            state,
            fieldname::EDGE_ENERGY_ELECTRIC,
            &energy,
            &energy_unit,
        )?;
        state_model.add_energy(
            state,
            fieldname::TRIP_ENERGY_ELECTRIC,
            &energy,
            &energy_unit,
        )?;
        let end_soc = energy_model_ops::update_soc_percent(
            &start_soc,
            (&energy, &energy_unit),
            (battery_capacity.0, battery_capacity.1),
        )?;
        state_model.set_custom_f64(state, fieldname::TRIP_SOC, &end_soc)?;
    } else {
        // use liquid fuel model
        let (energy, energy_unit) = charge_sustaining_model.predict(speed, grade, distance)?;
        state_model.set_energy(state, fieldname::EDGE_ENERGY_LIQUID, &energy, &energy_unit)?;
        state_model.add_energy(state, fieldname::TRIP_ENERGY_LIQUID, &energy, &energy_unit)?;
    };
    Ok(())
}

#[cfg(test)]
mod test {
    use super::PhevEnergyModel;
    use crate::model::{
        fieldname,
        phev_energy_model::phev_traversal,
        prediction::{ModelType, PredictionModelConfig, PredictionModelRecord},
    };
    use itertools::Itertools;
    use routee_compass_core::{
        model::{
            state::{StateModel, StateVariable},
            traversal::TraversalModel,
            unit::{
                AsF64, Distance, DistanceUnit, Energy, EnergyRate, EnergyRateUnit, EnergyUnit,
                Grade, GradeUnit, Speed, SpeedUnit,
            },
        },
        test::mock::traversal_model::TestTraversalModel,
    };
    use std::{path::PathBuf, sync::Arc};

    #[test]
    fn test_phev_energy_model_just_electric() {
        let (bat_cap, bat_unit) = (Energy::from(12.0), EnergyUnit::KilowattHours);
        let charge_depleting = mock_prediction_model("2016_CHEVROLET_Volt_Charge_Depleting");
        let charge_sustaining = mock_prediction_model("2016_CHEVROLET_Volt_Charge_Sustaining");
        let model = mock_phev(
            charge_sustaining.clone(),
            charge_depleting.clone(),
            100.0,
            (&bat_cap, &bat_unit),
        );
        let state_model = state_model(model);
        // starting at 100% SOC, we should be able to traverse 1000 meters
        // without using any liquid_fuel
        let distance = (Distance::from(1000.0), DistanceUnit::Meters);
        let speed = (Speed::from(60.0), SpeedUnit::MPH);
        let grade = (Grade::from(0.0), GradeUnit::Decimal);
        let mut state = state_vector(&state_model, distance, speed, grade);

        phev_traversal(
            &mut state,
            &state_model,
            charge_depleting.clone(),
            charge_sustaining.clone(),
            (&bat_cap, &bat_unit),
        )
        .expect("test invariant failed");

        let (elec, _) = state_model
            .get_energy(
                &state,
                fieldname::EDGE_ENERGY_ELECTRIC,
                Some(&EnergyUnit::KilowattHours),
            )
            .expect("test invariant failed");
        let (liquid, _) = state_model
            .get_energy(
                &state,
                fieldname::EDGE_ENERGY_LIQUID,
                Some(&EnergyUnit::GallonsGasoline),
            )
            .expect("test invariant failed");

        let soc = state_model
            .get_custom_f64(&state, fieldname::TRIP_SOC)
            .expect("test invariant failed");
        assert!(elec.as_f64() > 0.0, "elec energy {} should be > 0", elec);
        assert!(
            liquid.as_f64() < 1e-9,
            "liquid energy {} should be miniscule, < {}",
            liquid,
            1e-9
        );

        assert!(soc < 100.0, "soc {} should be < 100%", soc);
    }

    #[test]
    fn test_phev_energy_model_gas_and_electric() {
        let (bat_cap, bat_unit) = (Energy::from(12.0), EnergyUnit::KilowattHours);
        let charge_depleting = mock_prediction_model("2016_CHEVROLET_Volt_Charge_Depleting");
        let charge_sustaining = mock_prediction_model("2016_CHEVROLET_Volt_Charge_Sustaining");
        let model = mock_phev(
            charge_sustaining.clone(),
            charge_depleting.clone(),
            100.0,
            (&bat_cap, &bat_unit),
        );
        let state_model = state_model(model);

        // now let's traverse a really long link to deplete the battery
        let distance = (Distance::from(100.0), DistanceUnit::Miles);
        let speed = (Speed::from(60.0), SpeedUnit::MPH);
        let grade = (Grade::from(0.0), GradeUnit::Decimal);
        let mut state = state_vector(&state_model, distance, speed, grade);

        phev_traversal(
            &mut state,
            &state_model,
            charge_depleting.clone(),
            charge_sustaining.clone(),
            (&bat_cap, &bat_unit),
        )
        .expect("test invariant failed");

        let (elec, _) = state_model
            .get_energy(
                &state,
                fieldname::EDGE_ENERGY_ELECTRIC,
                Some(&EnergyUnit::KilowattHours),
            )
            .expect("test invariant failed");
        let (liquid, _) = state_model
            .get_energy(
                &state,
                fieldname::EDGE_ENERGY_LIQUID,
                Some(&EnergyUnit::GallonsGasoline),
            )
            .expect("test invariant failed");

        let soc = state_model
            .get_custom_f64(&state, fieldname::TRIP_SOC)
            .expect("test invariant failed");

        assert!(elec > Energy::ZERO, "elec energy {} should be > 0", elec);
        assert!(soc < 1e-9, "soc {} should be miniscule, < {}", soc, 1e-9);
        assert!(liquid == Energy::ZERO, "should not have used liquid energy");

        // and then traverse the same distance but this time we should only use liquid_fuel energy
        phev_traversal(
            &mut state,
            &state_model,
            charge_depleting.clone(),
            charge_sustaining.clone(),
            (&bat_cap, &bat_unit),
        )
        .expect("test invariant failed");

        let (liquid_energy_2, _) = state_model
            .get_energy(
                &state,
                fieldname::EDGE_ENERGY_LIQUID,
                Some(&EnergyUnit::GallonsGasoline),
            )
            .expect("test invariant failed");

        assert!(liquid_energy_2 > Energy::ZERO);
    }

    fn mock_phev(
        charge_sustaining: Arc<PredictionModelRecord>,
        charge_depleting: Arc<PredictionModelRecord>,
        starting_soc_percent: f64,
        battery_capacity: (&Energy, &EnergyUnit),
    ) -> Arc<dyn TraversalModel> {
        let bat_cap = *battery_capacity.0;
        let bat_unit = *battery_capacity.1;
        let staring_battery_energy: Energy =
            Energy::from(bat_cap.as_f64() * (starting_soc_percent * 0.01));

        let bev = PhevEnergyModel::new(
            charge_sustaining,
            charge_depleting,
            (bat_cap, bat_unit),
            (staring_battery_energy, bat_unit),
        )
        .expect("test invariant failed");

        // mock the upstream models via TestTraversalModel

        (TestTraversalModel::new(Arc::new(bev)).expect("test invariant failed")) as _
    }

    fn mock_prediction_model(model_name: &str) -> Arc<PredictionModelRecord> {
        let model_file_path: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("model")
            .join("test")
            .join(format!("{}.bin", &model_name));
        let model_filename = model_file_path.to_str().expect("test invariant failed");

        let model_config = PredictionModelConfig::new(
            model_name.to_string(),
            model_filename.to_string(),
            ModelType::Interpolate {
                underlying_model_type: Box::new(ModelType::Smartcore),
                speed_lower_bound: Speed::from(0.0),
                speed_upper_bound: Speed::from(100.0),
                speed_bins: 101,
                grade_lower_bound: Grade::from(-0.20),
                grade_upper_bound: Grade::from(0.20),
                grade_bins: 41,
            },
            SpeedUnit::MPH,
            GradeUnit::Decimal,
            EnergyRateUnit::KWHPM,
            Some(EnergyRate::from(0.2)),
            Some(1.3958),
            None,
        );
        let model_record =
            PredictionModelRecord::try_from(&model_config).expect("test invariant failed");
        Arc::new(model_record)
    }

    fn state_model(m: Arc<dyn TraversalModel>) -> StateModel {
        let out_f = m.output_features().into_iter().map(|(n, _)| n).join(", ");
        println!("output features: [{}]", out_f);
        let state_model = StateModel::empty()
            .register(m.input_features(), m.output_features())
            .expect("test invariant failed");
        println!(
            "registered state model features: {:?}",
            state_model.to_vec()
        );
        state_model
    }

    fn state_vector(
        state_model: &StateModel,
        distance: (Distance, DistanceUnit),
        speed: (Speed, SpeedUnit),
        grade: (Grade, GradeUnit),
    ) -> Vec<StateVariable> {
        let mut state = state_model.initial_state().unwrap();
        state_model
            .set_distance(
                &mut state,
                fieldname::EDGE_DISTANCE,
                &distance.0,
                &distance.1,
            )
            .expect("test invariant failed");
        state_model
            .set_speed(&mut state, fieldname::EDGE_SPEED, &speed.0, &speed.1)
            .expect("test invariant failed");
        state_model
            .set_grade(&mut state, fieldname::EDGE_GRADE, &grade.0, &grade.1)
            .expect("test invariant failed");
        state
    }
}
