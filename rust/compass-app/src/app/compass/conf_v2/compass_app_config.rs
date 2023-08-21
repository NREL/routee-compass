use std::collections::HashMap;

use super::{
    traversal_model::distance_builder::DistanceBuilder,
    traversal_model_builder::TraversalModelBuilder,
};

pub struct CompassAppConfig2 {
    tm_builders: HashMap<String, Box<dyn TraversalModelBuilder>>,
}

impl CompassAppConfig2 {
    // pub fn add_tm_builder(&mut self, name: String, builder: Box<dyn TraversalModelBuilder>) {
    //     match self.tm_builders.insert(name.clone(), builder) {
    //         Some(_prev) => log::warn!("adding traversal model builder {} more than once", &name),
    //         None => (),
    //     }
    // }

    pub fn default() -> CompassAppConfig2 {
        let dist: Box<dyn TraversalModelBuilder> = Box::new(DistanceBuilder {});
        let tms: HashMap<String, Box<dyn TraversalModelBuilder>> =
            HashMap::from([(String::from("distance"), dist)]);
        return CompassAppConfig2 { tm_builders: tms };
    }
}

#[cfg(test)]
mod tests {

    use std::path::PathBuf;

    use config::{Config, ConfigError, File, FileFormat};

    use crate::app::compass::conf_v2::compass_app_config::CompassAppConfig2;

    #[test]
    fn test_parse() {
        let toml_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("app")
            .join("compass")
            .join("conf_v2")
            .join("test")
            .join("conf.toml");
        let toml_conf = Config::builder()
            .add_source(config::File::from(toml_file))
            .build()
            .unwrap();
        let compass_app_config = CompassAppConfig2::default();
        let traversal_params = toml_conf.get::<serde_json::Value>("traversal").unwrap();
        println!("{:?}", toml_conf.get::<serde_json::Value>("traversal"));
        let tm_type_obj = traversal_params.get("type").unwrap();
        let tm_type = String::from(tm_type_obj.as_str().unwrap());
        println!("type: {}", tm_type);
        let tm_builder = compass_app_config.tm_builders.get(&tm_type).unwrap();
        let tm = tm_builder.build(&traversal_params).unwrap();
        let init_state = tm.initial_state();
        println!("distance init state: {:?}", init_state);
    }
}
