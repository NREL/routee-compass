use routee_compass_core::model::unit::{as_f64::AsF64, Energy};

pub fn as_soc_percent(current_energy: &Energy, max_energy: &Energy) -> f64 {
    let percent = (current_energy.as_f64() / max_energy.as_f64()) * 100.0;
    percent.max(0.0).min(100.0)
}
