# Compass Powertrain Crate

This crate provides energy-aware route planning models that integrate [RouteE Powertrain](https://github.com/nrel/routee-powertrain) into RouteE Compass.

### Energy Estimation in RouteE Compass

A RouteE Powertrain is an energy estimation model trained on NREL's [FASTSim](https://www.nrel.gov/transportation/fastsim.html) model.
RouteE Powertrain is an ML model which makes it sufficiently fast to run within the loop of a route planner cost model.
RouteE Powertrain models are trained and exported via the RouteE Powertrain utility and then loaded into a runtime at the core of this crate.

### Model Runtimes

There are two underlying model runtimes available, [smartcore](https://smartcorelib.org/) and [ort](https://github.com/pykeio/ort) (for [ONNX](https://onnx.ai/) models).
By default, this crate is loaded with ONNX deactivated. 
To activate the ONNX feature, pass the `onnx` feature flag during compilation.
Runtime kernels for 3 common OSs have been provided in the onnx-runtime directory within this crate.
For more information on cargo features, see The Cargo Book chapter on [Features](https://doc.rust-lang.org/cargo/reference/features.html).

The runtime is loaded via the TraversalModel(s) in this crate and used to estimate costs in RouteE Compass searches.

### Usage

The TraversalModel in this crate is integrated into [compass-app](../compass-app/README.md) and can be installed by running compass-app with a TraversalModel that uses energy.
An example traversal model configuration that uses this crate may look like this:

```toml
[traversal]
type = "speed_grade_energy_model"
model_type = "smartcore"
speed_table_input_file = "edges-posted-speed-enumerated.txt.gz"
energy_model_input_file = "2016_TOYOTA_Camry_4cyl_2WD.bin"
ideal_energy_rate = 0.02857142857
speed_table_speed_unit = "kilometers_per_hour"
energy_model_speed_unit = "miles_per_hour"
energy_model_grade_unit = "decimal"
energy_model_energy_rate_unit = "gallons_gasoline_per_mile"
output_time_unit = "minutes"
output_distance_unit = "miles"
grade_table_input_file = "edges-grade-enumerated.txt.gz"
grade_table_grade_unit = "decimal"
```

This TOML section is deserialized into JSON and passed as arguments to the SpeedGradeEnergyModelBuilder which in turn loads the [SpeedGradeModelService] in this crate.
This in turn builds the [SpeedGradeModel].

### Search

TraversalModels in this crate will add energy estimation to road network search, and will differ in their dependencies and evaluation procedures. 

#### SpeedGradeModel

##### Dependencies

- speeds per graph edge as a lookup table
- grade per graph edge as a lookup table _(optional)_
- an `energy_cost_coefficient` value, _(optional with default of 1.0)_
- a `real_world_adjustment_factor` value, _(optional with default of 1.0)_

##### Evaluation

1. lookup speed for the edge in table
2. compute travel time as `distance / speed` for this edge
3. lookup grade for this edge in table; if missing, use `0.0`
4. perform inference to retrieve energy rate from speed and grade values
5. compute energy as `energy_rate * distance * real_world_adjustment_factor` for this edge 
6. compute link cost as `(energy * energy_cost_coefficient) + (time * (1 - energy_cost_coefficient))`

### Real-World Adjustment Factors

In addition to calculating the energy based on a RouteE Powertrain output, an adjustment factor should be applied to capture real-world effects of running a powertrain in an environment.
As a result of NREL research, some recommended values for this adjustment are:

powertrain type | factor
--- | ---
combustion vehicle (CV) | 1.1660
hybrid vehicle (HV) | 1.1252
electric vehicle (EV) | 1.3958

A factor of 1.0 equates to 100% of the original energy value.

[SpeedGradeModelService]: crate::routee::speed_grade_model_service::SpeedGradeModelService
[SpeedGradeModelService]: crate::routee::speed_grade_model::SpeedGradeModel