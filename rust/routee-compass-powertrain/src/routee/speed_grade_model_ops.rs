use routee_compass_core::model::traversal::traversal_model_error::TraversalModelError;

pub enum PowertrainType {
    ICE,
    BEV,
    PHEV,
}

impl PowertrainType {
    pub fn from_string(s: String) -> Result<PowertrainType, TraversalModelError> {
        match s.as_str() {
            "ICE" => Ok(PowertrainType::ICE),
            "BEV" => Ok(PowertrainType::BEV),
            "PHEV" => Ok(PowertrainType::PHEV),
            _ => Err(TraversalModelError::BuildError(format!(
                "Unknown powertrain type: {}. Try one of ICE, BEV, PHEV",
                s
            ))),
        }
    }
}
