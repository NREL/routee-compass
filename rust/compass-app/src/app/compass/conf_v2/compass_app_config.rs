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

    use crate::app::compass::conf_v2::compass_app_config::CompassAppConfig2;
    use config::{Config, ConfigError, File, FileFormat};
    use std::path::PathBuf;

    #[test]
    fn test_parse() {
        let toml_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("app")
            .join("compass")
            .join("conf_v2")
            .join("test")
            .join("conf.toml");

        // parse TOML, but not yet deserialized. later, we will grab nested bits as serde_json::Value
        let toml_conf = Config::builder()
            .add_source(config::File::from(toml_file))
            .build()
            .unwrap();

        // this is the thing with the builders hashmap. it has one entry for this tes
        // which is "distance" -> ThingThatBuildsDistanceTraversalModel
        let compass_app_config = CompassAppConfig2::default();

        // grab the traversal model params and type from the file
        let traversal_params = toml_conf.get::<serde_json::Value>("traversal").unwrap();
        println!("contents of 'traversal' section: {:?}", traversal_params);
        let tm_type_obj = traversal_params.get("type").unwrap();
        let tm_type = String::from(tm_type_obj.as_str().unwrap());
        println!("traversal model type: {}", tm_type);

        // build the traversal model
        let tm_builder = compass_app_config.tm_builders.get(&tm_type).unwrap();
        let tm = tm_builder.build(&traversal_params).unwrap();
        let init_state = tm.initial_state();
        println!("traversal init state: {:?}", init_state);
    }
}
