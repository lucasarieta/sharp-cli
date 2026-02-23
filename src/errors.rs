use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SharpError {
    #[error("failed to read file '{}': {}", .0.display(), .1)]
    IoError(PathBuf, #[source] std::io::Error),

    #[error("invalid YAML schema: {0}")]
    YamlError(#[from] serde_yaml::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn io_error_display_includes_path() {
        let err = SharpError::IoError(
            PathBuf::from("/tmp/missing.yaml"),
            std::io::Error::new(std::io::ErrorKind::NotFound, "not found"),
        );
        let msg = err.to_string();
        assert!(msg.contains("/tmp/missing.yaml"));
        assert!(msg.contains("not found"));
        assert!(msg.starts_with("failed to read file"));
    }

    #[test]
    fn yaml_error_display() {
        let yaml_err: Result<serde_yaml::Value, _> = serde_yaml::from_str("{{invalid");
        let err = SharpError::YamlError(yaml_err.unwrap_err());
        let msg = err.to_string();
        assert!(msg.starts_with("invalid YAML schema"));
    }

    #[test]
    fn yaml_error_from_conversion() {
        let yaml_err: Result<serde_yaml::Value, _> = serde_yaml::from_str("{{bad");
        let sharp_err: SharpError = yaml_err.unwrap_err().into();
        assert!(matches!(sharp_err, SharpError::YamlError(_)));
    }
}
