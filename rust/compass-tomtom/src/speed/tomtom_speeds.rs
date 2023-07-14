use super::netw_to_speed_profile::build_speed_profile_lookup;
use super::weekday::Weekday;
use compass_core::model::cost::cost::Cost;
use compass_core::model::traversal::function::cost_function_error::CostFunctionError;
use compass_core::model::traversal::function::function::EdgeCostFunction;
use compass_core::model::traversal::state::search_state::StateVector;
use compass_core::model::traversal::state::state_variable::StateVar;
use compass_core::model::units::cm_per_second::CmPerSecond;
use compass_core::model::units::seconds::Seconds;
use std::fs::File;

/// constructs a lookup table that gives us travel times by time-of-day
/// bin. the source data is stored in 5-minute bins. we use the start time,
/// plus all accumulated time in the traversal state to compute the time bin.
/// using the time bin and edge id, we look up the speed profile, etc, and
/// produce a speed value which can be used to get the estimated travel time.
///
/// # Arguments
pub fn from_edgelist_csv(
    netw2speed_profile_filename: String,
    speed_profile_filename: String,
    profile2speed_per_time_slot_filename: String,
    speed_per_time_slot_filename: String,
    bin_size: usize,
) -> Result<EdgeCostFunction, CostFunctionError> {
    // function to look up the time bin based on the current time
    let bin_fn = move |s: &StateVector| -> Result<usize, CostFunctionError> {
        let time = get_travel_time(s)?;
        let start_time = get_start_time(s)?;
        let time = s[0].0 as i64;
        let result = (start_time.0 as i64 + time) as usize / bin_size;
        Ok(result)
    };

    // https://docs.rs/csv/latest/csv/index.html#example-with-serde
    // todo: dump each record type into a Vec<Record> so we can look these up
    // the EdgeId class is also u64, so, the top-level search Vec is indexed by those
    let speed_profile_file = File::open(&speed_profile_filename).map_err(|e| {
        CostFunctionError::FileReadError(speed_profile_filename.clone(), e.to_string())
    })?;
    let speed_profile_lookup =
        build_speed_profile_lookup(speed_profile_file, is_gzip(&speed_profile_filename)).map_err(
            |e| CostFunctionError::FileReadError(speed_profile_filename.clone(), e.to_string()),
        )?;

    let f: EdgeCostFunction = Box::new(move |o, e, d, s| {
        // s.travel_time() ?

        let _bin = bin_fn(&s);
        let _edge_id = e.edge_id;
        // lookup speed profile by edge id, pick speed by time bin...
        // see https://github.nrel.gov/MBAP/mbap-computing/blob/master/postgres/examples/tomtom_2021_network/tomtom_2021_network_time_bin_speeds.sql
        // https://pages.github.nrel.gov/MBAP/tomtom_2021_docs/mnr_spec/common_spec/theme_roads_and_ferries/speed_profile/speed_profile.html?hl=speed%2Cprofile
        let speed: CmPerSecond = CmPerSecond(1);
        let tt = e.distance_centimeters.travel_time_millis(&speed);
        let mut s_update = s.to_vec();
        s_update[0] = s_update[0] + StateVar(tt.0 as f64);
        Ok((Cost(tt.0), s_update))
    });

    return Ok(f);
}

/// creates the initial state vector of a search that uses tomtom speeds
///
/// # Arguments
///
/// * `start_time_sec` - the time of day, in seconds, that the search started at
/// * `start_day` - the day of the week that the search started on
pub fn initial_tomtom_state(start_time_sec: Seconds, start_day: Weekday) -> StateVector {
    let days: StateVar = StateVar::ZERO;
    let ms: StateVar = StateVar(start_time_sec.to_milliseconds().0 as f64);
    let weekday: StateVar = StateVar(start_day.day_number() as f64);
    let init_vec = vec![days, weekday, ms, ms];
    init_vec
}

const DAYS_INDEX: usize = 0;
const WEEKDAY_INDEX: usize = 1;
const START_TIME_INDEX: usize = 2;
const TRAVEL_TIME_INDEX: usize = 3;

pub fn get_days(sv: &StateVector) -> Result<StateVar, CostFunctionError> {
    sv.get(DAYS_INDEX)
        .ok_or(CostFunctionError::StateVectorIndexOutOfBounds(
            DAYS_INDEX,
            String::from("days"),
            sv.clone(),
        ))
        .copied()
}
pub fn get_weekday(sv: &StateVector) -> Result<StateVar, CostFunctionError> {
    sv.get(WEEKDAY_INDEX)
        .ok_or(CostFunctionError::StateVectorIndexOutOfBounds(
            WEEKDAY_INDEX,
            String::from("weekday"),
            sv.clone(),
        ))
        .copied()
}
pub fn get_start_time(sv: &StateVector) -> Result<StateVar, CostFunctionError> {
    sv.get(START_TIME_INDEX)
        .ok_or(CostFunctionError::StateVectorIndexOutOfBounds(
            START_TIME_INDEX,
            String::from("start time"),
            sv.clone(),
        ))
        .copied()
}
pub fn get_travel_time(sv: &StateVector) -> Result<StateVar, CostFunctionError> {
    sv.get(TRAVEL_TIME_INDEX)
        .ok_or(CostFunctionError::StateVectorIndexOutOfBounds(
            TRAVEL_TIME_INDEX,
            String::from("travel time"),
            sv.clone(),
        ))
        .copied()
}

pub fn is_gzip(file: &String) -> bool {
    file.ends_with(".gz")
}
