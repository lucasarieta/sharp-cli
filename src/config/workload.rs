use crate::config::schema::EventSchema;

#[derive(Debug)]
pub struct WorkloadProfile {
    pub events_per_day: u64,
    pub multi_tenant: bool,
    pub retention_days: u32,
}

impl WorkloadProfile {
    pub fn from_schema(schema: &EventSchema) -> Self {
        Self {
            events_per_day: schema.event_table.expected_events_per_day,
            multi_tenant: schema.event_table.multi_tenant,
            retention_days: schema.event_table.retention_days,
        }
    }
}
