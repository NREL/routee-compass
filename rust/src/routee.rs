use onnxruntime::{
    environment::Environment,
    ndarray::{Array, Array2},
    tensor::OrtOwnedTensor, GraphOptimizationLevel, LoggingLevel,
};

use anyhow::Result;

pub fn predict_model() -> Result<f32> {
    // get the folder that has cargo.toml in it
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;

    let src_dir = std::path::Path::new(&manifest_dir).join("src").join("ice.onnx");

    let model_file = src_dir.to_str().unwrap();

    println!("model_file: {}", model_file);

    let env = Environment::builder()
        .with_log_level(LoggingLevel::Verbose)
        .build()?;

    let mut session = env
        .new_session_builder()?
        .with_optimization_level(GraphOptimizationLevel::Basic)?
        .with_model_from_file(model_file)?;

    let distance_miles = 1.0;
    let speed_mph = 40.0;
    let grade_deciaml = 0.0;

    let input_data: Array2<f32> = Array::from_shape_vec((1, 2), vec![speed_mph, grade_deciaml])?;
    let input_tensor = vec![input_data];
    let outputs: Vec<OrtOwnedTensor<f32, _>> = session.run(input_tensor)?;
    let rate = outputs[0].get(0).unwrap();
    let energy = rate * distance_miles;
    Ok(energy)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_predict_model() {
        let energy = predict_model().unwrap();
        println!("energy: {}", energy)
    }
}

