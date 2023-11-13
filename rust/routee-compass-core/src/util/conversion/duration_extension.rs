use super::conversion_error::ConversionError;
use regex::Regex;
use std::time::Duration;

pub trait DurationExtension {
    fn as_duration(&self) -> Result<Duration, ConversionError>;
}

impl DurationExtension for serde_json::Value {
    fn as_duration(&self) -> Result<Duration, ConversionError> {
        let d_str = self.as_str().ok_or(ConversionError::DecoderError(
            format!("{:?}", self),
            String::from("JSON"),
        ))?;

        // regex lib recommends not building these within-the-loop, so this is not
        // good for performance to build here, but ran out of time exploring alternatives.
        // luckily (famous last words), only planning to use this at initialization.
        let duration_regex: Regex = Regex::new(r"^(?<h>\d+):(?<m>\d{2}):(?<s>\d{2})$")?;
        let Some(group) = duration_regex.captures(d_str) else {
            return Err(ConversionError::DecoderError(
                format!("{:?}", self),
                String::from("JSON"),
            ));
        };

        let h = &group["h"].parse::<u64>()?;
        let m = &group["m"].parse::<u64>()?;
        let s = &group["s"].parse::<u64>()?;
        Ok(Duration::from_secs(h * 3600 + m * 60 + s))
    }
}
