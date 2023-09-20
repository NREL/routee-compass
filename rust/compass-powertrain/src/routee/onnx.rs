#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ndarray::{Array1, CowArray};
    use ort::{Environment, GraphOptimizationLevel, SessionBuilder, Value, tensor::OrtOwnedTensor};

    #[test]
    fn test_load() {
        let model_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("routee")
            .join("test")
            .join("Toyota_Camry.onnx");

        let env = Environment::builder().build().unwrap().into_arc();

        let session = SessionBuilder::new(&env)
            .unwrap()
            .with_intra_threads(1)
            .unwrap()
            .with_optimization_level(GraphOptimizationLevel::Level1)
            .unwrap()
            .with_model_from_file(model_file_path)
            .unwrap();

        let x = CowArray::from(
            Array1::from(vec![60.0 as f32, 0.0 as f32])
                .into_shape((1, 2))
                .unwrap(),
        )
        .into_dyn();

        let input = vec![Value::from_array(session.allocator(), &x).unwrap()];

        let result: OrtOwnedTensor<f32, _> = session.run(input).unwrap()[0].try_extract().unwrap();
        let energy_per_mile = result.view().to_owned().into_raw_vec()[0] as f64;
        println!("energy_per_mile: {:?}", energy_per_mile);
    }
}
