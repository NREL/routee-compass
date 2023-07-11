use compass_core::model::cost::cost::Cost;
use compass_core::model::traversal::function::function::EdgeCostFunction;
use compass_core::model::traversal::state::search_state::StateVector;
use compass_core::model::traversal::state::state_variable::StateVar;
use compass_core::model::units::cm_per_second::CmPerSecond;
use std::fs::File;
// use csv;

/// constructs a lookup table that gives us travel times by time-of-day
/// bin. the source data is stored in 5-minute bins. we use the start time,
/// plus all accumulated time in the traversal state to compute the time bin.
/// using the time bin and edge id, we look up the speed profile, etc, and
/// produce a speed value which can be used to get the estimated travel time.

pub fn from_edgelist_csv(start_time: i64, file: &File, bin_size: i64) -> EdgeCostFunction {
    let bin_fn = move |s: &StateVector| -> i64 {
        let time = s[0].0 as i64;
        (start_time + time) / bin_size
    };

    // https://docs.rs/csv/latest/csv/index.html#example-with-serde
    // todo: dump each record type into a Vec<Record> so we can look these up
    // the EdgeId class is also u64, so, the top-level search Vec is indexed by those
    csv::Reader::from_reader(file);

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

    return f;
}
