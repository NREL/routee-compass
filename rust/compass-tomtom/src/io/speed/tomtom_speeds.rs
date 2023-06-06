use compass_core::io::lookup_table::EdgeLookupTable;
use compass_core::model::cost::cost::Cost;
use compass_core::model::traversal::state::has_travel_time::HasTravelTime;
use compass_core::model::units::cm_per_second::CmPerSecond;
use std::fs::File;
// use csv;

/// constructs a lookup table that gives us travel times by time-of-day
/// bin. the source data is stored in 5-minute bins. we use the start time,
/// plus all accumulated time in the traversal state to compute the time bin.
/// using the time bin and edge id, we look up the speed profile, etc, and
/// produce a speed value which can be used to get the estimated travel time.

pub fn from_edgelist_csv<S>(start_time: Cost, file: &File, bin_size: Cost) -> EdgeLookupTable<S>
where
    S: HasTravelTime,
{
    let bin_fn = move |c: Cost| -> i64 { (start_time.0 + c.0) / bin_size.0 };

    // https://docs.rs/csv/latest/csv/index.html#example-with-serde
    // todo: dump each record type into a Vec<Record> so we can look these up
    // the EdgeId class is also u64, so, the top-level search Vec is indexed by those
    csv::Reader::from_reader(file);

    let f: EdgeLookupTable<S> = Box::new(move |(o, e, d, s)| {
        // s.travel_time() ?
        let bin = bin_fn(s.travel_time());
        let edge_id = e.edge_id;
        // lookup speed profile by edge id, pick speed by time bin...
        // see https://github.nrel.gov/MBAP/mbap-computing/blob/master/postgres/examples/tomtom_2021_network/tomtom_2021_network_time_bin_speeds.sql
        // https://pages.github.nrel.gov/MBAP/tomtom_2021_docs/mnr_spec/common_spec/theme_roads_and_ferries/speed_profile/speed_profile.html?hl=speed%2Cprofile
        let speed: CmPerSecond = CmPerSecond(1);
        let tt = e.distance_centimeters.travel_time_millis(&speed);
        Ok(Cost(tt.0))
    });
    todo!()
}
