use anyhow::Result;
use std::fs;
use std::path::Path;

const LOG_FILE: &str = ".jpkg/last_error.log";

pub fn log_error(error: &str) -> Result<()> {
    fs::create_dir_all(".jpkg")?;
    fs::write(LOG_FILE, error)?;
    Ok(())
}

pub fn get_last_error() -> Result<String> {
    if !Path::new(LOG_FILE).exists() {
        return Ok("No errors logged yet.".to_string());
    }
    Ok(fs::read_to_string(LOG_FILE)?)
}

#[allow(dead_code)]
pub fn clear_error_log() -> Result<()> {
    if Path::new(LOG_FILE).exists() {
        fs::remove_file(LOG_FILE)?;
    }
    Ok(())
}
