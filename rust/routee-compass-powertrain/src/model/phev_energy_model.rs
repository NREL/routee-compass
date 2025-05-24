use crate::model::fieldname;

use super::{
    energy_model_ops,
    prediction::{PredictionModelConfig, PredictionModelRecord},
};
use routee_compass_core::model::{
    network::{Edge, Vertex},
    state::{CustomFeatureFormat, InputFeature, OutputFeature, StateModel, StateVariable},
    traversal::{TraversalModel, TraversalModelError, TraversalModelService},
    unit::{AsF64, Convert, Distance, Energy, EnergyUnit, UnitError},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{borrow::Cow, sync::Arc};

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
        let liq_unit = self
            .charge_sustain_model
            .energy_rate_unit
            .associated_energy_unit();
        let ele_unit = self
            .charge_depleting_model
            .energy_rate_unit
            .associated_energy_unit();

        vec![
            (
                String::from(fieldname::TRIP_ENERGY),
                OutputFeature::Energy {
                    energy_unit: EnergyUnit::GallonsGasolineEquivalent,
                    initial: Energy::ZERO,
                    accumulator: true,
                },
            ),
            (
                String::from(fieldname::EDGE_ENERGY),
                OutputFeature::Energy {
                    energy_unit: EnergyUnit::GallonsGasolineEquivalent,
                    initial: Energy::ZERO,
                    accumulator: false,
                },
            ),
            (
                String::from(fieldname::TRIP_ENERGY_LIQUID),
                OutputFeature::Energy {
                    energy_unit: liq_unit,
                    initial: Energy::ZERO,
                    accumulator: true,
                },
            ),
            (
                String::from(fieldname::EDGE_ENERGY_LIQUID),
                OutputFeature::Energy {
                    energy_unit: liq_unit,
                    initial: Energy::ZERO,
                    accumulator: false,
                },
            ),
            (
                String::from(fieldname::TRIP_ENERGY_ELECTRIC),
                OutputFeature::Energy {
                    energy_unit: ele_unit,
                    initial: Energy::ZERO,
                    accumulator: true,
                },
            ),
            (
                String::from(fieldname::EDGE_ENERGY_ELECTRIC),
                OutputFeature::Energy {
                    energy_unit: ele_unit,
                    initial: Energy::ZERO,
                    accumulator: false,
                },
            ),
            (
                String::from(fieldname::TRIP_SOC),
                OutputFeature::Custom {
                    name: String::from("soc"),
                    unit: String::from("Percent"),
                    format: CustomFeatureFormat::FloatingPoint {
                        initial: self.starting_soc.into(),
                    },
                    accumulator: true,
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
            false,
        )
    }

    /// estimates energy use based only on using the liquid fuel model
    fn estimate_traversal(
        &self,
        _od: (&Vertex, &Vertex),
        state: &mut Vec<StateVariable>,
        state_model: &StateModel,
    ) -> Result<(), TraversalModelError> {
        phev_traversal(
            state,
            state_model,
            self.charge_depleting_model.clone(),
            self.charge_sustain_model.clone(),
            (&self.battery_capacity.0, &self.battery_capacity.1),
            true,
        )
    }
}

/// a PHEV traversal that switches between a charge depleting and charge sustaining model in order
/// to compute energy consumption. the mechanism checks for battery state of charge, and if it is
/// greater than 0, then the charge depleting model is used. otherwise, the charge sustaining model is used.
/// if battery state is empty mid-traversal, the remaining distance will be applied to the charge
/// sustaining model.
fn phev_traversal(
    state: &mut [StateVariable],
    state_model: &StateModel,
    depleting: Arc<PredictionModelRecord>,
    sustaining: Arc<PredictionModelRecord>,
    battery_capacity: (&Energy, &EnergyUnit),
    estimate: bool,
) -> Result<(), TraversalModelError> {
    // collect state variables
    let (battery_cap, battery_unit) = battery_capacity;
    let (distance, distance_unit) =
        state_model.get_distance(state, fieldname::EDGE_DISTANCE, None)?;
    let start_soc = state_model.get_custom_f64(state, fieldname::TRIP_SOC)?;
    let (trip_energy_elec, trip_elec_unit) =
        state_model.get_energy(state, fieldname::TRIP_ENERGY_ELECTRIC, None)?;

    // figure out how much energy we had at the start of the edge
    let mut elec_used = Cow::Owned(trip_energy_elec);
    trip_elec_unit.convert(&mut elec_used, battery_unit)?;
    let edge_start_elec = *battery_cap - elec_used.into_owned();

    // estimate remaining energy if we travel this distance
    let (est_edge_elec, est_elec_unit) = if estimate {
        Energy::create(
            (&distance, distance_unit),
            (&depleting.ideal_energy_rate, &depleting.energy_rate_unit),
        )?
    } else {
        let speed = state_model.get_speed(state, fieldname::EDGE_SPEED, None)?;
        let grade = state_model.get_grade(state, fieldname::EDGE_GRADE, None)?;
        depleting.predict(speed, grade, (distance, distance_unit))?
    };

    let mut est_edge_elec = Cow::Owned(est_edge_elec);
    est_elec_unit.convert(&mut est_edge_elec, battery_unit)?;
    let est_edge_elec = est_edge_elec.into_owned();
    let remaining_elec = edge_start_elec - est_edge_elec;

    // did we complete the edge on battery or do we need to switch to our charge sustaining model?
    let energy_overage = Energy::ZERO - remaining_elec;
    let completed_on_battery = energy_overage <= Energy::ZERO;
    if completed_on_battery {
        depleting_only_traversal(
            state,
            state_model,
            start_soc,
            &est_edge_elec,
            battery_cap,
            battery_unit,
        )
    } else {
        mixed_traversal(
            state,
            state_model,
            sustaining,
            &edge_start_elec,
            &trip_energy_elec,
            &energy_overage,
            battery_unit,
            estimate,
        )
    }
}

/// used when edge traversal can be fully served by battery fuel
fn depleting_only_traversal(
    state: &mut [StateVariable],
    state_model: &StateModel,
    start_soc: f64,
    est_edge_elec: &Energy,
    battery_capacity: &Energy,
    battery_unit: &EnergyUnit,
) -> Result<(), TraversalModelError> {
    state_model.set_energy(
        state,
        fieldname::EDGE_ENERGY_ELECTRIC,
        est_edge_elec,
        battery_unit,
    )?;
    state_model.add_energy(
        state,
        fieldname::TRIP_ENERGY_ELECTRIC,
        est_edge_elec,
        battery_unit,
    )?;
    // update trip energy in GGE
    let gge = accumulate_gge(&[(est_edge_elec, battery_unit)])?;
    state_model.set_energy(
        state,
        fieldname::EDGE_ENERGY,
        &gge,
        &EnergyUnit::GallonsGasolineEquivalent,
    )?;
    state_model.add_energy(
        state,
        fieldname::TRIP_ENERGY,
        &gge,
        &EnergyUnit::GallonsGasolineEquivalent,
    )?;
    let end_soc = energy_model_ops::update_soc_percent(
        &start_soc,
        (est_edge_elec, battery_unit),
        (battery_capacity, battery_unit),
    )?;
    state_model.set_custom_f64(state, fieldname::TRIP_SOC, &end_soc)?;
    Ok(())
}

///
fn mixed_traversal(
    state: &mut [StateVariable],
    state_model: &StateModel,
    sustaining: Arc<PredictionModelRecord>,
    edge_start_elec: &Energy,
    trip_energy_elec: &Energy,
    energy_overage: &Energy,
    battery_unit: &EnergyUnit,
    estimate: bool,
) -> Result<(), TraversalModelError> {
    let (distance, distance_unit) =
        state_model.get_distance(state, fieldname::EDGE_DISTANCE, None)?;

    // use up remaining battery first (edge start electricity, not edge consumption electricity)
    state_model.set_energy(
        state,
        fieldname::EDGE_ENERGY_ELECTRIC,
        edge_start_elec,
        battery_unit,
    )?;
    state_model.add_energy(
        state,
        fieldname::TRIP_ENERGY_ELECTRIC,
        edge_start_elec,
        battery_unit,
    )?;
    state_model.set_custom_f64(state, fieldname::TRIP_SOC, &0.0)?;

    // find the amount of distance remaining on this edge as a ratio of remaining energy to total energy used
    let numer = trip_energy_elec.as_f64();
    let denom = (*trip_energy_elec + *energy_overage).as_f64();
    let remaining_ratio = 1.0 - (numer / denom);
    let remaining_dist = Distance::from(distance.as_f64() * remaining_ratio);

    // estimate energy over this distance at the ideal energy rate for the charge sustaining model
    // estimate remaining energy if we travel this distance
    let (remaining_energy, remaining_unit) = if estimate {
        Energy::create(
            (&remaining_dist, distance_unit),
            (&sustaining.ideal_energy_rate, &sustaining.energy_rate_unit),
        )?
    } else {
        let speed = state_model.get_speed(state, fieldname::EDGE_SPEED, None)?;
        let grade = state_model.get_grade(state, fieldname::EDGE_GRADE, None)?;
        sustaining.predict(speed, grade, (remaining_dist, distance_unit))?
    };

    state_model.set_energy(
        state,
        fieldname::EDGE_ENERGY_LIQUID,
        &remaining_energy,
        &remaining_unit,
    )?;
    state_model.add_energy(
        state,
        fieldname::TRIP_ENERGY_LIQUID,
        &remaining_energy,
        &remaining_unit,
    )?;
    // update trip energy in GGE from both depleting and sustaining phases
    let gge = accumulate_gge(&[
        (edge_start_elec, battery_unit),
        (&remaining_energy, &remaining_unit),
    ])?;
    state_model.set_energy(
        state,
        fieldname::EDGE_ENERGY,
        &gge,
        &EnergyUnit::GallonsGasolineEquivalent,
    )?;
    state_model.add_energy(
        state,
        fieldname::TRIP_ENERGY,
        &gge,
        &EnergyUnit::GallonsGasolineEquivalent,
    )?;
    Ok(())
}

/// helper function to accumulate a variety of energy observations into a single GGE energy value.
fn accumulate_gge(values: &[(&Energy, &EnergyUnit)]) -> Result<Energy, UnitError> {
    let mut acc = Energy::ZERO;
    for (energy, energy_unit) in values.iter() {
        let mut conv = Cow::Borrowed(*energy);
        energy_unit.convert(&mut conv, &EnergyUnit::GallonsGasolineEquivalent)?;
        acc = acc + conv.into_owned();
    }
    Ok(acc)
}

#[cfg(test)]
mod test {
    use super::PhevEnergyModel;
    use crate::model::{
        fieldname,
        phev_energy_model::phev_traversal,
        prediction::{ModelType, PredictionModelConfig, PredictionModelRecord},
    };
    use routee_compass_core::{
        model::{
            state::{StateModel, StateVariable},
            traversal::TraversalModel,
            unit::{
                AsF64, Distance, DistanceUnit, Energy, EnergyRateUnit, EnergyUnit, Grade,
                GradeUnit, Speed, SpeedUnit,
            },
        },
        testing::mock::traversal_model::TestTraversalModel,
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
            false,
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
            false,
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

        let (liquid_if_no_electricity_used, _) = Energy::create(
            (&distance.0, &distance.1),
            (
                &charge_sustaining.ideal_energy_rate,
                &charge_sustaining.energy_rate_unit,
            ),
        )
        .expect("failed to create ideal liquid energy");

        assert!(
            elec == bat_cap,
            "elec energy {} should be == battery capacity {}",
            elec,
            bat_cap
        );
        assert!(soc == 0.0, "soc {} should be 0", soc);
        assert!(liquid < liquid_if_no_electricity_used, "liquid energy {} should have been less than the amount if we only used liquid energy {}", liquid, liquid_if_no_electricity_used);

        // and then traverse the same distance but this time we should only use liquid_fuel energy
        phev_traversal(
            &mut state,
            &state_model,
            charge_depleting.clone(),
            charge_sustaining.clone(),
            (&bat_cap, &bat_unit),
            false,
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

        println!(
            "{:?}",
            serde_json::to_string_pretty(&state_model.serialize_state(&state)).unwrap()
        );
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
            Some(1.3958),
            None,
        );
        let model_record =
            PredictionModelRecord::try_from(&model_config).expect("test invariant failed");
        Arc::new(model_record)
    }

    fn state_model(m: Arc<dyn TraversalModel>) -> StateModel {
        StateModel::empty()
            .register(m.input_features(), m.output_features())
            .expect("test invariant failed")
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
