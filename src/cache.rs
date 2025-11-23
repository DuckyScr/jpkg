use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

/// Get the cache directory path (~/.jpkg/cache/)
pub fn cache_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    Ok(home.join(".jpkg").join("cache"))
}

/// Initialize cache directory
pub fn init_cache() -> Result<()> {
    let cache = cache_dir()?;
    fs::create_dir_all(&cache)?;
    Ok(())
}

/// Get cached JAR path for a specific artifact
pub fn get_cached_jar(group: &str, artifact: &str, version: &str) -> Result<Option<PathBuf>> {
    let cache = cache_dir()?;
    let filename = format!("{}-{}.jar", artifact, version);
    let cached_path = cache
        .join(group.replace('.', "/"))
        .join(artifact)
        .join(version)
        .join(&filename);

    if cached_path.exists() {
        Ok(Some(cached_path))
    } else {
        Ok(None)
    }
}

/// Cache a JAR file
pub fn cache_jar(group: &str, artifact: &str, version: &str, jar_path: &Path) -> Result<()> {
    let cache = cache_dir()?;
    let filename = format!("{}-{}.jar", artifact, version);
    let cache_path = cache
        .join(group.replace('.', "/"))
        .join(artifact)
        .join(version);

    fs::create_dir_all(&cache_path)?;
    let dest = cache_path.join(&filename);

    if !dest.exists() {
        fs::copy(jar_path, dest)?;
    }

    Ok(())
}

/// List all cached artifacts
pub fn list_cached() -> Result<Vec<String>> {
    let cache = cache_dir()?;
    let mut artifacts = Vec::new();

    if !cache.exists() {
        return Ok(artifacts);
    }

    visit_cache_dir(&cache, &cache, &mut artifacts)?;
    Ok(artifacts)
}

fn visit_cache_dir(dir: &Path, cache_root: &Path, artifacts: &mut Vec<String>) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            visit_cache_dir(&path, cache_root, artifacts)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("jar") {
            // Extract artifact info from path
            if let Ok(rel_path) = path.strip_prefix(cache_root) {
                let parts: Vec<_> = rel_path.components().collect();
                if parts.len() >= 3 {
                    let group = parts[..parts.len() - 3]
                        .iter()
                        .map(|c| c.as_os_str().to_string_lossy())
                        .collect::<Vec<_>>()
                        .join(".");
                    let artifact = parts[parts.len() - 3].as_os_str().to_string_lossy();
                    let version = parts[parts.len() - 2].as_os_str().to_string_lossy();
                    artifacts.push(format!("{}:{}:{}", group, artifact, version));
                }
            }
        }
    }

    Ok(())
}

/// Clear the cache
pub fn clear_cache() -> Result<()> {
    let cache = cache_dir()?;
    if cache.exists() {
        fs::remove_dir_all(&cache)?;
    }
    init_cache()?;
    Ok(())
}

/// Get cache size in bytes
pub fn cache_size() -> Result<u64> {
    let cache = cache_dir()?;
    if !cache.exists() {
        return Ok(0);
    }

    let mut total = 0u64;
    visit_cache_size(&cache, &mut total)?;
    Ok(total)
}

fn visit_cache_size(dir: &Path, total: &mut u64) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            visit_cache_size(&path, total)?;
        } else {
            *total += entry.metadata()?.len();
        }
    }

    Ok(())
}
