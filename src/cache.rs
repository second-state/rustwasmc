//! Getting and configuring ssvmup's binary cache.

use binary_install::Cache;
use std::env;
use std::path::Path;

/// Get ssvmup's binary cache.
pub fn get_ssvmup_cache() -> Result<Cache, failure::Error> {
    if let Ok(path) = env::var("SSVMUP_CACHE") {
        Ok(Cache::at(Path::new(&path)))
    } else {
        Cache::new("ssvmup")
    }
}
