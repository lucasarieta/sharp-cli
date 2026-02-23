pub mod schema;
pub mod workload;

use crate::errors::SharpError;
use schema::EventSchema;
use std::path::Path;

pub fn load_schema(path: &Path) -> Result<EventSchema, SharpError> {
    let contents = std::fs::read_to_string(path)
        .map_err(|e| SharpError::IoError(path.to_path_buf(), e))?;
    let schema: EventSchema =
        serde_yaml::from_str(&contents).map_err(SharpError::YamlError)?;
    Ok(schema)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn loads_valid_schema_file() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        writeln!(
            tmp,
            "event_table:\n  name: clicks\n  multi_tenant: true\n  expected_events_per_day: 100000\n  retention_days: 30"
        )
        .unwrap();

        let schema = load_schema(tmp.path()).unwrap();
        assert_eq!(schema.event_table.name, "clicks");
        assert!(schema.event_table.multi_tenant);
        assert_eq!(schema.event_table.expected_events_per_day, 100_000);
        assert_eq!(schema.event_table.retention_days, 30);
    }

    #[test]
    fn returns_io_error_for_missing_file() {
        let result = load_schema(Path::new("/nonexistent/schema.yaml"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("failed to read file"));
    }

    #[test]
    fn returns_yaml_error_for_invalid_content() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        writeln!(tmp, "not: valid: yaml: [[[").unwrap();

        let result = load_schema(tmp.path());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("invalid YAML schema"));
    }

    #[test]
    fn multi_tenant_defaults_to_false() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        writeln!(
            tmp,
            "event_table:\n  name: events\n  expected_events_per_day: 1000\n  retention_days: 7"
        )
        .unwrap();

        let schema = load_schema(tmp.path()).unwrap();
        assert!(!schema.event_table.multi_tenant);
    }
}
