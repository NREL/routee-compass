use super::road_class_service::RoadClassFrontierService;
use crate::config::{CompassConfigurationField, ConfigJsonExtensions};
use crate::{
    model::constraint::{ConstraintModelBuilder, ConstraintModelError, ConstraintModelService},
    util::fs::{read_decoders, read_utils},
};
use kdam::Bar;
use std::{collections::HashMap, sync::Arc};

pub struct RoadClassBuilder {}

impl ConstraintModelBuilder for RoadClassBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn ConstraintModelService>, ConstraintModelError> {
        let constraint_key = CompassConfigurationField::Constraint.to_string();
        let road_class_file_key = String::from("road_class_input_file");

        let road_class_file = parameters
            .get_config_path(&road_class_file_key, &constraint_key)
            .map_err(|e| {
                ConstraintModelError::BuildError(format!(
                    "configuration error due to {}: {}",
                    road_class_file_key.clone(),
                    e
                ))
            })?;

        let road_class_lookup: Box<[String]> = read_utils::read_raw_file(
            &road_class_file,
            read_decoders::string,
            Some(Bar::builder().desc("road class")),
            None,
        )
        .map_err(|e| {
            ConstraintModelError::BuildError(format!(
                "failed to load file at {:?}: {}",
                road_class_file.clone().to_str(),
                e
            ))
        })?;

        let mut mapping = HashMap::new();
        let mut encoded = Vec::with_capacity(road_class_lookup.len());
        let mut next_id = 0usize;

        for class in road_class_lookup.iter() {
            let id = match mapping.get(class) {
                Some(id) => *id,
                None => {
                    let id_usize = next_id;
                    if id_usize > u8::MAX as usize {
                        return Err(ConstraintModelError::BuildError(
                            "too many unique road classes, max is 256".to_string(),
                        ));
                    }
                    next_id += 1;
                    let id = id_usize as u8;
                    mapping.insert(class.clone(), id);
                    id
                }
            };
            encoded.push(id);
        }

        let m: Arc<dyn ConstraintModelService> = Arc::new(RoadClassFrontierService {
            road_class_by_edge: Arc::new(encoded.into_boxed_slice()),
            road_class_mapping: Arc::new(mapping),
        });
        Ok(m)
    }
}
