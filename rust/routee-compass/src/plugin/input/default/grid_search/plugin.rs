use crate::plugin::input::input_field::InputField;
use crate::plugin::input::input_json_extensions::InputJsonExtensions;
use crate::plugin::input::input_plugin::InputPlugin;
use crate::plugin::plugin_error::PluginError as E;
use routee_compass_core::util::multiset::MultiSet;

/// Builds an input plugin that duplicates queries if array-valued fields are present
/// by stepping through each combination of value
pub struct GridSearchPlugin {}

impl InputPlugin for GridSearchPlugin {
    fn process(&self, input: &serde_json::Value) -> Result<Vec<serde_json::Value>, E> {
        match input.get_grid_search() {
            None => Ok(vec![input.clone()]),
            Some(grid_search_input) => {
                let map = grid_search_input
                    .as_object()
                    .ok_or(E::UnexpectedQueryStructure(format!("{:?}", input)))?;
                let mut keys: Vec<String> = vec![];
                let mut multiset_input: Vec<Vec<serde_json::Value>> = vec![];
                let mut multiset_indices: Vec<Vec<usize>> = vec![];
                for (k, v) in map {
                    if let Some(v) = v.as_array() {
                        keys.push(k.to_string());
                        multiset_input.push(v.to_vec());
                        let indices = (0..v.len()).collect();
                        multiset_indices.push(indices);
                    }
                }
                // for each combination, copy the grid search values into a fresh
                // copy of the source (minus the "grid_search" key)
                // let remove_key = InputField::GridSearch.to_str();
                let mut initial_map = input
                    .as_object()
                    .ok_or(E::UnexpectedQueryStructure(format!("{:?}", input)))?
                    .clone();
                initial_map.remove(InputField::GridSearch.to_str());
                let initial = serde_json::json!(initial_map);
                let multiset = MultiSet::from(&multiset_indices);
                let result: Vec<serde_json::Value> = multiset
                    .into_iter()
                    .map(|combination| {
                        let mut instance = initial.clone();
                        let it = keys.iter().zip(combination.iter()).enumerate();
                        for (set_idx, (key, val_idx)) in it {
                            let value = multiset_input[set_idx][*val_idx].clone();
                            match value {
                                serde_json::Value::Object(o) => {
                                    for (k, v) in o.into_iter() {
                                        instance[k] = v.clone();
                                    }
                                }
                                _ => {
                                    instance[key] = multiset_input[set_idx][*val_idx].clone();
                                }
                            }
                        }
                        instance
                    })
                    .collect();

                Ok(result)
            }
        }
    }
}

#[cfg(test)]
mod test {

    use super::GridSearchPlugin;
    use crate::plugin::input::input_plugin::InputPlugin;

    #[test]
    fn test_grid_search_empty_parent_object() {
        let input = serde_json::json!({
            "grid_search": {
                "bar": ["a", "b", "c"],
                "foo": [1.2, 3.4]
            }
        });
        let plugin = GridSearchPlugin {};
        let result = plugin
            .process(&input)
            .unwrap()
            .iter()
            .map(serde_json::to_string)
            .collect::<Result<Vec<String>, serde_json::Error>>()
            .unwrap();
        let expected = vec![
            String::from("{\"bar\":\"a\",\"foo\":1.2}"),
            String::from("{\"bar\":\"b\",\"foo\":1.2}"),
            String::from("{\"bar\":\"c\",\"foo\":1.2}"),
            String::from("{\"bar\":\"a\",\"foo\":3.4}"),
            String::from("{\"bar\":\"b\",\"foo\":3.4}"),
            String::from("{\"bar\":\"c\",\"foo\":3.4}"),
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_grid_search_persisted_parent_keys() {
        let input = serde_json::json!({
            "ignored_key": "ignored_value",
            "grid_search": {
                "bar": ["a", "b", "c"],
                "foo": [1.2, 3.4]
            }
        });
        let plugin = GridSearchPlugin {};
        let result = plugin
            .process(&input)
            .unwrap()
            .iter()
            .map(serde_json::to_string)
            .collect::<Result<Vec<String>, serde_json::Error>>()
            .unwrap();
        let expected = vec![
            String::from("{\"bar\":\"a\",\"foo\":1.2,\"ignored_key\":\"ignored_value\"}"),
            String::from("{\"bar\":\"b\",\"foo\":1.2,\"ignored_key\":\"ignored_value\"}"),
            String::from("{\"bar\":\"c\",\"foo\":1.2,\"ignored_key\":\"ignored_value\"}"),
            String::from("{\"bar\":\"a\",\"foo\":3.4,\"ignored_key\":\"ignored_value\"}"),
            String::from("{\"bar\":\"b\",\"foo\":3.4,\"ignored_key\":\"ignored_value\"}"),
            String::from("{\"bar\":\"c\",\"foo\":3.4,\"ignored_key\":\"ignored_value\"}"),
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_grid_search_using_objects() {
        let input = serde_json::json!({
            "ignored_key": "ignored_value",
            "grid_search": {
                "a": [1, 2],
                "ignored_inner_key": [
                    { "x": 0, "y": 0 },
                    { "x": 1, "y": 1 }
                ],
            }
        });
        let plugin = GridSearchPlugin {};
        let result = plugin
            .process(&input)
            .unwrap()
            .iter()
            .map(serde_json::to_string)
            .collect::<Result<Vec<String>, serde_json::Error>>()
            .unwrap();
        let expected = vec![
            String::from("{\"a\":1,\"ignored_key\":\"ignored_value\",\"x\":0,\"y\":0}"),
            String::from("{\"a\":2,\"ignored_key\":\"ignored_value\",\"x\":0,\"y\":0}"),
            String::from("{\"a\":1,\"ignored_key\":\"ignored_value\",\"x\":1,\"y\":1}"),
            String::from("{\"a\":2,\"ignored_key\":\"ignored_value\",\"x\":1,\"y\":1}"),
        ];
        assert_eq!(result, expected);
    }
}
