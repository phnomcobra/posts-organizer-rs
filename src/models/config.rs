use serde_json::json;

use crate::dao::kvstore::KVStore;

/// Log level, mirroring Python's `LogLevel` IntEnum.
#[derive(Clone, Debug, PartialEq, Default)]
pub enum LogLevel {
    Debug,
    #[default]
    Info,
    Warning,
    Error,
    Critical,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warning => write!(f, "WARNING"),
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Critical => write!(f, "CRITICAL"),
        }
    }
}

impl std::str::FromStr for LogLevel {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "DEBUG" => Ok(LogLevel::Debug),
            "INFO" => Ok(LogLevel::Info),
            "WARNING" | "WARN" => Ok(LogLevel::Warning),
            "ERROR" => Ok(LogLevel::Error),
            "CRITICAL" => Ok(LogLevel::Critical),
            _ => Err(format!(
                "unknown log level '{s}'; expected DEBUG, INFO, WARNING, ERROR, or CRITICAL"
            )),
        }
    }
}

/// Configuration settings for the posts organizer application.
/// All values are loaded from the kvstore collection; defaults are seeded on first run.
#[derive(Clone, Debug)]
pub struct Config {
    /// Number of worker processes (default: 2).
    pub workers: usize,
    /// IP address or hostname to bind the HTTP server (default: 0.0.0.0).
    pub socket_host: String,
    /// Port number for the HTTP server (default: 8080).
    pub socket_port: u16,
    /// Log level for the application logger (default: INFO).
    pub log_level_app: LogLevel,
    /// Log level for the API/web framework logger (default: WARNING).
    pub log_level_api: LogLevel,
    /// Seconds before a post is discarded (default: 600).
    pub timeout_secs: u32,
}

impl Config {
    /// Load all config values from the kvstore, seeding defaults for any missing keys.
    pub fn load() -> Self {
        let store = KVStore::new(None).expect("failed to open kvstore");

        macro_rules! kv_str {
            ($key:expr, $default:expr) => {
                store
                    .get($key, Some(json!($default)))
                    .unwrap_or(json!($default))
                    .as_str()
                    .unwrap_or($default)
                    .to_string()
            };
        }
        macro_rules! kv_u32 {
            ($key:expr, $default:expr) => {
                store
                    .get($key, Some(json!($default)))
                    .unwrap_or(json!($default))
                    .as_u64()
                    .unwrap_or($default as u64) as u32
            };
        }
        macro_rules! kv_usize {
            ($key:expr, $default:expr) => {
                store
                    .get($key, Some(json!($default)))
                    .unwrap_or(json!($default))
                    .as_u64()
                    .unwrap_or($default as u64) as usize
            };
        }
        macro_rules! kv_level {
            ($key:expr, $default:expr) => {{
                let s = store
                    .get($key, Some(json!($default)))
                    .unwrap_or(json!($default))
                    .as_str()
                    .unwrap_or($default)
                    .to_string();
                s.parse::<LogLevel>().unwrap_or_default()
            }};
        }

        let socket_port = store
            .get("socket_port", Some(json!(8080u16)))
            .unwrap_or(json!(8080u16))
            .as_u64()
            .unwrap_or(8080) as u16;

        Self {
            workers: kv_usize!("workers", 2usize),
            socket_host: kv_str!("socket_host", "0.0.0.0"),
            socket_port,
            log_level_app: kv_level!("log_level_app", "Info"),
            log_level_api: kv_level!("log_level_api", "Info"),
            timeout_secs: kv_u32!("timeout_secs", 600u32),
        }
    }

    /// Log the current configuration values.
    pub fn log(&self) {
        log::info!(
            "\n  workers:              {}\n  socket_host:          {}\n  socket_port:          {}\n  log_level_app:        {}\n  log_level_api:        {}\n  timeout_secs:         {}",
            self.workers, self.socket_host, self.socket_port,
            self.log_level_app, self.log_level_api,
            self.timeout_secs,
        );
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::load()
    }
}
