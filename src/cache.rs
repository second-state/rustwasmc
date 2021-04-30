//! Getting and configuring rustwasmc's binary cache.

use binary_install::Cache;
use std::env;
use std::path::Path;

/// Get rustwasmc's binary cache.
pub fn get_rustwasmc_cache() -> Result<Cache, failure::Error> {
    if let Ok(path) = env::var("RUSTWASMC_CACHE") {
        Ok(Cache::at(Path::new(&path)))
    } else {
        Cache::new("rustwasmc")
    }
}
