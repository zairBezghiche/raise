pub mod config;
pub mod error;
pub mod logger;

// Re-exports pour un usage simplifié
pub use config::AppConfig;
pub use error::{AppError, Result};
pub use logger::init_logging;
