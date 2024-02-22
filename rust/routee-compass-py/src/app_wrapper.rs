use routee_compass::app::compass::{
    compass_app::CompassApp, config::compass_app_builder::CompassAppBuilder,
};
use routee_compass_macros::pybindings;

#[pybindings]
pub struct CompassAppWrapper {
    pub app: CompassApp,
}


impl CompassAppBindings for CompassAppWrapper {
    fn new(app: CompassApp) -> Self {
        CompassAppWrapper { app }
    }

    fn app(&self) -> &CompassApp {
        &self.app
    }
}