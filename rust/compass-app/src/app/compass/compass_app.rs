use crate::{
    app::{app_error::AppError, search::search_app::SearchApp},
    plugin::{input::InputPlugin, output::OutputPlugin},
};

pub struct CompassApp<'app> {
    pub search_app: &'app SearchApp<'app>,
    pub input_plugins: Vec<&'app InputPlugin>,
    pub output_plugins: Vec<&'app OutputPlugin>,
}

impl<'app> CompassApp<'app> {
    pub fn run_vertex_oriented(
        &self,
        query: serde_json::Value,
    ) -> Result<serde_json::Value, AppError> {
        // 1. reads the JSON query as
        //   - a single request object,
        //   - an array of request objects
        //   - produces an array of JSON objects
        // 2. applies the input plugins to each query in the array
        // 3. runs the search app
        // 4. applies the output plugins to each search result
        // 5. returns the JSON result
        todo!()
    }
}
