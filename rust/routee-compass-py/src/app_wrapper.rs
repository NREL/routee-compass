use routee_compass::app::{
    bindings::CompassAppBindings,
    compass::{compass_app::CompassApp, CompassAppBuilder, CompassAppError},
};
use routee_compass_macros::pybindings;

#[pybindings]
pub struct CompassAppWrapper {
    pub app: CompassApp,
}

impl CompassAppBindings for CompassAppWrapper {
    fn from_config_toml_string(
        config_string: String,
        original_file_path: String,
    ) -> Result<Self, CompassAppError>
    where
        Self: Sized,
    {
        let builder = CompassAppBuilder::default();
        let app =
            CompassApp::try_from_config_toml_string(config_string, original_file_path, &builder)?;
        Ok(CompassAppWrapper { app })
    }
    fn app(&self) -> &CompassApp {
        &self.app
    }
}
