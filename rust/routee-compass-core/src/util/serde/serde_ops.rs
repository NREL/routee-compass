use serde::de;

/// hack-ish trick for types which can be deserialized from a string
/// representation, such as enums where all variants have no arguments.
///
/// # Arguments
///
/// * `input` - string to deserialize
///
/// # Returns
///
/// the deserialized value or a deserialization error
pub fn string_deserialize<T>(input: &str) -> Result<T, serde_json::Error>
where
    T: de::DeserializeOwned,
{
    let mut enquoted = input.to_owned();
    enquoted.insert(0, '"');
    enquoted.push('"');
    serde_json::from_str::<T>(enquoted.as_str())
}
