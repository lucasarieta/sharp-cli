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
