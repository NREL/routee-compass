use super::road_map_error::RoadMapError;

pub trait RoadMap {
    type RoadMapResult;
    fn route(
        &self,
        origin: [isize; 2],
        destination: [isize; 2],
    ) -> Result<Self::RoadMapResult, RoadMapError>;
}
