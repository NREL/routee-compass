use super::{
    vehicle_parameter::VehicleParameter,
    vehicle_restriction_service::VehicleRestrictionFrontierService,
};
use routee_compass_core::{
    algorithm::search::{Direction, SearchTreeBranch},
    model::{
        frontier::{FrontierModel, FrontierModelError},
        network::{Edge, VertexId},
        state::{StateModel, StateVariable},
    },
};
use std::{collections::HashMap, sync::Arc};

pub struct VehicleRestrictionFrontierModel {
    pub service: Arc<VehicleRestrictionFrontierService>,
    pub vehicle_parameters: Vec<VehicleParameter>,
}

impl FrontierModel for VehicleRestrictionFrontierModel {
    fn valid_frontier(
        &self,
        edge: &Edge,
        _state: &[StateVariable],
        _tree: &HashMap<VertexId, SearchTreeBranch>,
        _direction: &Direction,
        _state_model: &StateModel,
    ) -> Result<bool, FrontierModelError> {
        validate_edge(self, edge)
    }

    fn valid_edge(&self, edge: &Edge) -> Result<bool, FrontierModelError> {
        validate_edge(self, edge)
    }
}

fn validate_edge(
    model: &VehicleRestrictionFrontierModel,
    edge: &Edge,
) -> Result<bool, FrontierModelError> {
    match model.service.vehicle_restriction_lookup.get(&edge.edge_id) {
        None => Ok(true),
        Some(vehicle_restrictions) => {
            let valid =
                model
                    .vehicle_parameters
                    .iter()
                    .all(|vehicle_parameter| {
                        match vehicle_restrictions.get(&vehicle_parameter.name()) {
                            Some(restriction) => restriction.validate_parameters(vehicle_parameter),
                            None => true,
                        }
                    });
            Ok(valid)
        }
    }
}

#[cfg(test)]
mod test {

    use crate::app::compass::model::frontier_model::vehicle_restrictions::*;
    use routee_compass_core::model::{
        frontier::{FrontierModel, FrontierModelBuilder},
        network::Edge,
        state::StateModel,
    };
    use serde_json::{json, Value};
    use std::{
        path::{Path, PathBuf},
        sync::Arc,
    };

    #[test]
    fn test_e2e_valid_weight_and_height() {
        let model = build_model("test_restrictions.csv", "unrestricted.json");
        let edge = Edge::new(0, 0, 0, 0.0);
        match model.valid_edge(&edge) {
            Ok(is_ok) => assert!(is_ok),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn test_e2e_invalid_weight() {
        let model = build_model("test_restrictions.csv", "overweight.json");
        let edge = Edge::new(0, 0, 0, 0.0);
        match model.valid_edge(&edge) {
            Ok(is_ok) => assert!(!is_ok),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn test_e2e_invalid_height() {
        let model = build_model("test_restrictions.csv", "overheight.json");
        let edge = Edge::new(0, 0, 0, 0.0);
        match model.valid_edge(&edge) {
            Ok(is_ok) => assert!(!is_ok),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn test_e2e_unrestricted_parameter() {
        let model = build_model("test_restrictions.csv", "unknown_parameter.json");
        let edge = Edge::new(0, 0, 0, 0.0);
        match model.valid_edge(&edge) {
            Ok(is_ok) => assert!(is_ok),
            Err(e) => panic!("{}", e),
        }
    }

    fn build_model(restriction_filename: &str, query_filename: &str) -> Arc<dyn FrontierModel> {
        let restriction_file = test_filepath(restriction_filename);
        let conf = json!({
            "vehicle_restriction_input_file": restriction_file,
        });
        let query = read_json_file(query_filename);
        let service = VehicleRestrictionBuilder {}
            .build(&conf)
            .unwrap_or_else(|e| {
                panic!(
                    "failed to read test CSV {} due to: {}",
                    restriction_filename, e
                )
            });
        let state_model = Arc::new(StateModel::new(vec![]));

        (service.build(&query, state_model).unwrap_or_else(|_| {
            panic!(
                "failed to build model from service with query: {}",
                &serde_json::to_string(&query).unwrap_or_default()
            )
        })) as _
    }

    fn read_json_file(filename: &str) -> Value {
        let filepath = test_filepath(filename);
        let file_contents = std::fs::read_to_string(&filepath)
            .unwrap_or_else(|_| panic!("test invariant failed, unable to load {}", &filepath));

        serde_json::from_str(&file_contents)
            .unwrap_or_else(|_| panic!("test invariant failed, unable to parse {}", &filepath))
    }

    fn test_filepath(filename: &str) -> String {
        let mut path = test_dir();
        path.push(filename);
        path.to_str()
            .unwrap_or_else(|| panic!("test invariant failed, unable to load {}", filename))
            .to_string()
    }

    fn test_dir() -> PathBuf {
        // rust/routee-compass/src/app/compass/model/frontier_model/vehicle_restrictions/test
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("app")
            .join("compass")
            .join("model")
            .join("frontier_model")
            .join("vehicle_restrictions")
            .join("test")
    }
}
