use super::netw_to_speed_profile::build_speed_profile_lookup;
use super::weekday::Weekday;
use compass_core::model::cost::cost::Cost;
use compass_core::model::property::edge::Edge;
use compass_core::model::property::vertex::Vertex;
use compass_core::model::traversal::state::state_variable::StateVar;
use compass_core::model::traversal::state::traversal_state::TraversalState;
use compass_core::model::traversal::traversal_model::TraversalModel;
use compass_core::model::traversal::traversal_model_error::TraversalModelError;
use compass_core::model::traversal::traversal_result::TraversalResult;
use compass_core::util::unit::Time;
use std::fs::File;
use uom::si;

pub struct TomTomSpeedLookup {
    bin_size: usize,
    start_time_seconds: f64,
    start_day: Weekday,
}

impl TraversalModel for TomTomSpeedLookup {
    fn initial_state(&self) -> TraversalState {
        let days: StateVar = StateVar::ZERO;
        let ms: StateVar = StateVar(self.start_time_seconds);
        let weekday: StateVar = StateVar(self.start_day.day_number() as f64);
        let init_vec = vec![days, weekday, ms, ms];
        init_vec
    }

    fn traversal_cost(
        &self,
        src: &Vertex,
        edge: &Edge,
        dst: &Vertex,
        state: &TraversalState,
    ) -> Result<TraversalResult, TraversalModelError> {
        let _bin = bin_function(&state, self.bin_size);
        let _edge_id = edge.edge_id;
        // lookup speed profile by edge id, pick speed by time bin...
        // see https://github.nrel.gov/MBAP/mbap-computing/blob/master/postgres/examples/tomtom_2021_network/tomtom_2021_network_time_bin_speeds.sql
        // https://pages.github.nrel.gov/MBAP/tomtom_2021_docs/mnr_spec/common_spec/theme_roads_and_ferries/speed_profile/speed_profile.html?hl=speed%2Cprofile
        let tt = Time::new(1.0);
        let milliseconds = 0.001;
        let mut s_update = state.clone();
        s_update[0] = s_update[0] + StateVar(milliseconds as f64);
        let result = TraversalResult {
            total_cost: Cost::from(milliseconds),
            updated_state: s_update,
        };
        Ok(result)
    }

    fn summary(&self, state: &TraversalState) -> serde_json::Value {
        let time = state[0].0;
        let time_units = "milliseconds";
        serde_json::json!({
            "total_time": time,
            "units": time_units,
        })
    }

    fn cost_estimate(
        &self,
        src: &Vertex,
        dst: &Vertex,
        state: &TraversalState,
    ) -> Result<Cost, TraversalModelError> {
        todo!("not yet implemented")
    }
}
impl TomTomSpeedLookup {
    pub fn from_edgelist_csv(
        netw2speed_profile_filename: String,
        speed_profile_filename: String,
        profile2speed_per_time_slot_filename: String,
        speed_per_time_slot_filename: String,
        bin_size: usize,
        start_time_seconds: f64,
        start_day: Weekday,
    ) -> Result<Self, TraversalModelError> {
        // https://docs.rs/csv/latest/csv/index.html#example-with-serde
        // todo: dump each record type into a Vec<Record> so we can look these up
        // the EdgeId class is also u64, so, the top-level search Vec is indexed by those
        let speed_profile_file = File::open(&speed_profile_filename).map_err(|e| {
            TraversalModelError::FileReadError(speed_profile_filename.clone(), e.to_string())
        })?;
        let speed_profile_lookup =
            build_speed_profile_lookup(speed_profile_file, is_gzip(&speed_profile_filename))
                .map_err(|e| {
                    TraversalModelError::FileReadError(
                        speed_profile_filename.clone(),
                        e.to_string(),
                    )
                })?;

        let lookup = TomTomSpeedLookup {
            bin_size,
            start_time_seconds,
            start_day,
        };

        return Ok(lookup);
    }
}

pub fn bin_function(
    search_state: &TraversalState,
    bin_size: usize,
) -> Result<usize, TraversalModelError> {
    let time = get_travel_time(search_state)?;
    let start_time = get_start_time(search_state)?;
    let time = search_state[0].0 as i64;
    let result = (start_time.0 as i64 + time) as usize / bin_size;
    Ok(result)
}

const DAYS_INDEX: usize = 0;
const WEEKDAY_INDEX: usize = 1;
const START_TIME_INDEX: usize = 2;
const TRAVEL_TIME_INDEX: usize = 3;

pub fn get_days(sv: &TraversalState) -> Result<StateVar, TraversalModelError> {
    sv.get(DAYS_INDEX)
        .ok_or(TraversalModelError::StateVectorIndexOutOfBounds(
            DAYS_INDEX,
            String::from("days"),
            sv.clone(),
        ))
        .copied()
}
pub fn get_weekday(sv: &TraversalState) -> Result<StateVar, TraversalModelError> {
    sv.get(WEEKDAY_INDEX)
        .ok_or(TraversalModelError::StateVectorIndexOutOfBounds(
            WEEKDAY_INDEX,
            String::from("weekday"),
            sv.clone(),
        ))
        .copied()
}
pub fn get_start_time(sv: &TraversalState) -> Result<StateVar, TraversalModelError> {
    sv.get(START_TIME_INDEX)
        .ok_or(TraversalModelError::StateVectorIndexOutOfBounds(
            START_TIME_INDEX,
            String::from("start time"),
            sv.clone(),
        ))
        .copied()
}
pub fn get_travel_time(sv: &TraversalState) -> Result<StateVar, TraversalModelError> {
    sv.get(TRAVEL_TIME_INDEX)
        .ok_or(TraversalModelError::StateVectorIndexOutOfBounds(
            TRAVEL_TIME_INDEX,
            String::from("travel time"),
            sv.clone(),
        ))
        .copied()
}

pub fn update_days(mut sv: TraversalState, value: StateVar) {
    sv[DAYS_INDEX] = value;
}
pub fn update_weekday(mut sv: TraversalState, value: StateVar) {
    sv[WEEKDAY_INDEX] = value;
}
pub fn update_travel_time(mut sv: TraversalState, value: StateVar) {
    sv[TRAVEL_TIME_INDEX] = value;
}

pub fn is_gzip(file: &String) -> bool {
    file.ends_with(".gz")
}
