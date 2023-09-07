// use crate::plugin::input::input_json_extensions::InputJsonExtensions;
use crate::plugin::input::input_plugin::InputPlugin;
use crate::plugin::plugin_error::PluginError as E;
use compass_core::util::multiset::MultiSet;

/// Builds an input plugin that duplicates queries if array-valued fields are present
/// by stepping through each combination of value
pub struct GridSearchPlugin {}

impl InputPlugin for GridSearchPlugin {
    fn process(&self, input: &serde_json::Value) -> Result<Vec<serde_json::Value>, E> {
        let map = input
            .as_object()
            .ok_or(E::UnexpectedQueryStructure(format!("{:?}", input)))?;
        let mut keys: Vec<String> = vec![];
        let mut multiset_input: Vec<Vec<serde_json::Value>> = vec![];
        let mut multiset_indices: Vec<Vec<usize>> = vec![];
        for (k, v) in map {
            match v {
                serde_json::Value::Array(values) => {
                    keys.push(k.to_string());
                    multiset_input.push(values.to_vec());
                    let indices = (0..values.len()).collect();
                    multiset_indices.push(indices);
                }
                _ => {}
            }
        }
        let multiset = MultiSet::from(&multiset_indices);
        let result: Vec<serde_json::Value> = multiset
            .into_iter()
            .map(|selected| {
                let mut instance = input.clone();
                for (set_idx, (key, val_idx)) in keys.iter().zip(selected.iter()).enumerate() {
                    instance[key] = multiset_input[set_idx][*val_idx].clone();
                }
                instance
            })
            .collect();
        // https://docs.rs/itertools/latest/itertools/structs/struct.Permutations.html

        Ok(result)
    }
}

#[cfg(test)]
mod test {

    use super::GridSearchPlugin;
    use crate::plugin::input::input_plugin::InputPlugin;

    #[test]
    fn test_grid_search() {
        let input = serde_json::json!({
            "bar": ["a", "b", "c"],
            "foo": [1.2, 3.4]
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
}
