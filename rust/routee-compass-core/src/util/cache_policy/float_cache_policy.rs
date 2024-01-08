use std::{num::NonZeroUsize, sync::Mutex};

use lru::LruCache;
use serde::{Deserialize, Serialize};

use super::cache_error::CacheError;

fn to_precision(value: f64, precision: i32) -> i64 {
    let multiplier = 10f64.powi(precision);
    (value * multiplier).round() as i64
}

#[derive(Serialize, Deserialize)]
pub struct FloatCachePolicyConfig {
    pub cache_size: usize,
    pub key_precisions: Vec<i32>,
}

/// A cache policy that uses a float key to store a float value
/// The key is rounded to the specified precision.
///
/// # Example
///
/// ```
/// use routee_compass_core::util::cache_policy::float_cache_policy::{FloatCachePolicy, FloatCachePolicyConfig};
/// use std::num::NonZeroUsize;
///
/// let config = FloatCachePolicyConfig {
///    cache_size: 100,
///    key_precisions: vec![2, 2],
/// };
///
/// let cache_policy = FloatCachePolicy::from_config(config).unwrap();
///
/// // stores keys as [123, 235]
/// cache_policy.update(&[1.234, 2.345], 3.456).unwrap();
///
/// let value = cache_policy.get(&[1.234, 2.345]).unwrap().unwrap();
/// assert_eq!(value, 3.456);
///
/// // 1.233 rounds to 123
/// let value = cache_policy.get(&[1.233, 2.345]).unwrap().unwrap();
/// assert_eq!(value, 3.456);
///
/// // 1.2 rounds to 120
/// let value = cache_policy.get(&[1.2, 2.345]).unwrap();
/// assert_eq!(value, None);
///
/// // 2.344 rounds to 234
/// let value = cache_policy.get(&[1.234, 2.344]).unwrap();
/// assert_eq!(value, None);
pub struct FloatCachePolicy {
    cache: Mutex<LruCache<Vec<i64>, f64>>,
    key_precisions: Vec<i32>,
}

impl FloatCachePolicy {
    pub fn from_config(config: FloatCachePolicyConfig) -> Result<Self, CacheError> {
        let size = NonZeroUsize::new(config.cache_size).ok_or_else(|| {
            CacheError::BuildError("maximum_cache_size must be greater than 0".to_string())
        })?;
        let cache = Mutex::new(LruCache::new(size));
        for precision in config.key_precisions.iter() {
            if (*precision > 10) || (*precision < -10) {
                return Err(CacheError::BuildError(
                    "key_precision must be betwee -10 and 10".to_string(),
                ));
            }
        }
        Ok(Self {
            cache,
            key_precisions: config.key_precisions,
        })
    }

    pub fn float_key_to_int_key(&self, key: &[f64]) -> Vec<i64> {
        key.iter()
            .zip(self.key_precisions.iter())
            .map(|(v, p)| to_precision(*v, *p))
            .collect()
    }

    pub fn get(&self, key: &[f64]) -> Result<Option<f64>, CacheError> {
        let int_key = self.float_key_to_int_key(key);
        let mut cache = self.cache.lock().map_err(|e| {
            CacheError::RuntimeError(format!("Could not get lock on cache due to {}", e))
        })?;
        Ok(cache.get(&int_key).copied())
    }

    pub fn update(&self, key: &[f64], value: f64) -> Result<(), CacheError> {
        let int_key = self.float_key_to_int_key(key);
        let mut cache = self.cache.lock().map_err(|e| {
            CacheError::RuntimeError(format!("Could not get lock on cache due to {}", e))
        })?;
        cache.put(int_key, value);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_float_cache_policy() {
        let config = FloatCachePolicyConfig {
            cache_size: 100,
            key_precisions: vec![1, 3],
        };

        let cache_policy = FloatCachePolicy::from_config(config).unwrap();

        // should store keys as [12, 2345]
        cache_policy.update(&[1.234, 2.345], 3.456).unwrap();

        // same in same out
        let value = cache_policy.get(&[1.234, 2.345]).unwrap().unwrap();
        assert_eq!(value, 3.456);

        // 1.15 rounds to 12
        let value = cache_policy.get(&[1.15, 2.345]).unwrap().unwrap();
        assert_eq!(value, 3.456);

        // 1.14 rounds to 11
        let value = cache_policy.get(&[1.14, 2.345]).unwrap();
        assert_eq!(value, None);

        // 2.344 rounds to 2344
        let value = cache_policy.get(&[1.234, 2.344]).unwrap();
        assert_eq!(value, None);
    }
}
