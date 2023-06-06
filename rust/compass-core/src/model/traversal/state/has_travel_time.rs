use crate::model::cost::cost::Cost;

pub trait HasTravelTime {
    fn travel_time(&self) -> Cost;
}
