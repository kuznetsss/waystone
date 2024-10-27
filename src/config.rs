use std::fmt::Display;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub upstream_servers: Vec<String>,
}

#[derive(Debug)]
pub enum ConfigError {
    ErrorReadingFile {
        path: std::path::PathBuf,
        error: std::io::Error,
    },
    ParseError(serde_yml::Error),
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::ErrorReadingFile { path, error } => write!(
                f,
                "Error reading config file '{}': {}",
                path.to_str().unwrap(),
                error
            ),
            ConfigError::ParseError(err) => write!(f, "Error parsing config file {}", err),
        }
    }
}

impl From<serde_yml::Error> for ConfigError {
    fn from(value: serde_yml::Error) -> Self {
        ConfigError::ParseError(value)
    }
}

impl Config {
    pub fn new(content: &str) -> Result<Config, ConfigError> {
        serde_yml::from_str(content).map_err(|e| e.into())
    }

    pub fn from_file(config_path: &std::path::Path) -> Result<Config, ConfigError> {
        let content =
            std::fs::read_to_string(config_path).map_err(|e| ConfigError::ErrorReadingFile {
                path: config_path.to_path_buf(),
                error: e,
            })?;
        Self::new(content.as_str())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    static VALID_YAML: &str = "
upstream_servers:
  - host1
  - host2
  - host3
";

    #[test]
    fn from_valid_yaml() {
        Config::new(VALID_YAML).unwrap();
    }

    #[test]
    fn from_invalid_yaml() {
        match Config::new("invalid yaml") {
            Err(ConfigError::ParseError(_)) => {}
            other => panic!("Unexpected error {:?}", other),
        }
    }

    #[test]
    fn from_valid_yaml_file() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        let path = file.path().to_owned();
        write!(file, "{}", VALID_YAML).unwrap();
        Config::from_file(&path).unwrap();
    }

    #[test]
    fn from_not_existing_file() {
        match Config::from_file(std::path::Path::new("some wrong path")) {
            Err(ConfigError::ErrorReadingFile { path: _, error: _ }) => {}
            other => panic!("Unexpected error {:?}", other),
        }
    }
}
