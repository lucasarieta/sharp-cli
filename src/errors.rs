use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SharpError {
    #[error("failed to read file '{}': {}", .0.display(), .1)]
    IoError(PathBuf, #[source] std::io::Error),

    #[error("invalid YAML schema: {0}")]
    YamlError(#[from] serde_yaml::Error),
}
