use routee_compass_core::algorithm::search::search_orientation::SearchOrientation;

use super::response::{
    response_output_policy::ResponseOutputPolicy,
    response_persistence_policy::ResponsePersistencePolicy,
};

pub struct CompassAppConfiguration {
    pub parallelism: usize,
    pub search_orientation: SearchOrientation,
    pub response_persistence_policy: ResponsePersistencePolicy,
    pub response_output_policy: ResponseOutputPolicy,
}

impl CompassAppConfiguration {
    pub fn new(
        parallelism: usize,
        search_orientation: SearchOrientation,
        response_persistence_policy: ResponsePersistencePolicy,
        response_output_policy: ResponseOutputPolicy,
    ) -> CompassAppConfiguration {
        CompassAppConfiguration {
            parallelism,
            search_orientation,
            response_persistence_policy,
            response_output_policy,
        }
    }
}
