use crate::app::compass::config::builder::compass_app_builder::CompassAppBuilder;

pub trait CompassAppBuildContext {
    fn init(&self) -> CompassAppBuilder;
}
