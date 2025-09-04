use routee_compass::app::{
    bindings::CompassAppBindings,
    compass::{CompassApp, CompassAppConfig, CompassAppError, CompassBuilderInventory},
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
        let builder = CompassBuilderInventory::new()?;
        let config = CompassAppConfig::from_str(
            &config_string,
            &original_file_path,
            config::FileFormat::Toml,
        )?;
        let app = CompassApp::new(&config, &builder)?;
        Ok(CompassAppWrapper { app })
    }
    fn app(&self) -> &CompassApp {
        &self.app
    }
}
