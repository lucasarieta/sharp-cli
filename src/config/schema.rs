use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct EventSchema {
    pub event_table: EventTable,
}

#[derive(Debug, Deserialize)]
pub struct EventTable {
    pub name: String,
    #[serde(default)]
    pub multi_tenant: bool,
    pub expected_events_per_day: u64,
    pub retention_days: u32,
}
