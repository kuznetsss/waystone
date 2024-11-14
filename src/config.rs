use std::{error::Error, fmt::Display};

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    pub upstream_servers: Vec<String>,

    #[serde(default = "Config::default_threads_number")]
    pub threads_number: usize,
}

#[derive(Deserialize, Debug)]
pub struct ServerConfig {
    #[serde(default = "ServerConfig::default_ip")]
    pub ip: String,

    #[serde(default)]
    pub port: u32,
}

#[derive(Debug)]
pub enum ConfigError {
    ErrorReadingFile {
        path: std::path::PathBuf,
        error: std::io::Error,
    },
    ParseError(serde_yml::Error),
    ValueError(Vec<String>),
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ConfigError::*;

        match self {
            ErrorReadingFile { path, error } => write!(
                f,
                "Error reading config file '{}': {}",
                path.to_str().unwrap(),
                error
            ),
            ParseError(err) => write!(f, "Error parsing config file: {}", err),
            ValueError(errs) => {
                writeln!(f, "Wrong value found in config:")?;
                errs.iter().try_for_each(|e| writeln!(f, "- {}", e))?;
                Ok(())
            }
        }
    }
}

impl Error for ConfigError {}

impl From<serde_yml::Error> for ConfigError {
    fn from(value: serde_yml::Error) -> Self {
        ConfigError::ParseError(value)
    }
}

impl Config {
    pub fn new(content: &str) -> Result<Config, ConfigError> {
        let config: Config =
            serde_yml::from_str(content).map_err(|e| -> ConfigError { e.into() })?;

        if let Some(err) = config.check() {
            return Err(err);
        }

        Ok(config)
    }

    pub fn from_file(config_path: &std::path::Path) -> Result<Config, ConfigError> {
        let content =
            std::fs::read_to_string(config_path).map_err(|e| ConfigError::ErrorReadingFile {
                path: config_path.to_path_buf(),
                error: e,
            })?;
        Self::new(content.as_str())
    }

    fn default_threads_number() -> usize {
        std::thread::available_parallelism().unwrap().get()
    }

    fn check(&self) -> Option<ConfigError> {
        let mut errors = Vec::new();
        if self.upstream_servers.is_empty() {
            errors.push(
                "'upstream_servers' is empty. Please, provide at least one upstream server."
                    .to_string(),
            );
        }
        if errors.is_empty() {
            None
        } else {
            Some(ConfigError::ValueError(errors))
        }
    }
}

impl ServerConfig {
    pub fn ip_port(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }

    fn default_ip() -> String {
        "0.0.0.0".to_string()
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            ip: ServerConfig::default_ip(),
            port: u32::default(),
        }
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
    fn threads_number_value() {
        let threads_number = 3333;
        let mut file = tempfile::NamedTempFile::new().unwrap();
        let path = file.path().to_owned();
        write!(file, "{}\nthreads_number: {}", VALID_YAML, threads_number).unwrap();
        let config = Config::from_file(&path).unwrap();
        assert_eq!(config.threads_number, threads_number);
    }

    #[test]
    fn from_not_existing_file() {
        match Config::from_file(std::path::Path::new("some wrong path")) {
            Err(ConfigError::ErrorReadingFile { path: _, error: _ }) => {}
            other => panic!("Unexpected error {:?}", other),
        }
    }
}
