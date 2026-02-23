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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::schema::{EventSchema, EventTable};

    #[test]
    fn from_schema_maps_all_fields() {
        let schema = EventSchema {
            event_table: EventTable {
                name: "events".to_string(),
                multi_tenant: true,
                expected_events_per_day: 42_000_000,
                retention_days: 60,
            },
        };
        let workload = WorkloadProfile::from_schema(&schema);

        assert_eq!(workload.events_per_day, 42_000_000);
        assert!(workload.multi_tenant);
        assert_eq!(workload.retention_days, 60);
    }

    #[test]
    fn from_schema_single_tenant_defaults() {
        let schema = EventSchema {
            event_table: EventTable {
                name: "t".to_string(),
                multi_tenant: false,
                expected_events_per_day: 1,
                retention_days: 0,
            },
        };
        let workload = WorkloadProfile::from_schema(&schema);

        assert_eq!(workload.events_per_day, 1);
        assert!(!workload.multi_tenant);
        assert_eq!(workload.retention_days, 0);
    }
}
