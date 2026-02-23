use crate::config::schema::EventSchema;
use crate::config::workload::WorkloadProfile;

const HIGH_VOLUME_THRESHOLD: u64 = 100_000_000;
const SHARDING_THRESHOLD: u64 = 500_000_000;
const TENANT_PROJECTION_THRESHOLD: u64 = 50_000_000;

pub fn analyze(_schema: &EventSchema, workload: &WorkloadProfile) -> Vec<String> {
    let mut recommendations = Vec::new();

    // Retention policy
    if workload.retention_days > 0 {
        recommendations.push(format!(
            "Enable TTL to auto-expire old data (`TTL timestamp + INTERVAL {} DAY`)",
            workload.retention_days
        ));
    } else {
        recommendations.push(
            "Warn: data will grow unbounded without a retention policy".to_string(),
        );
    }

    // High-throughput ingestion
    if workload.events_per_day >= HIGH_VOLUME_THRESHOLD {
        recommendations.push(
            "Enable `wide_parts_only` merge-tree setting for high-throughput ingestion".to_string(),
        );
    }

    // Sharding
    if workload.events_per_day >= SHARDING_THRESHOLD {
        recommendations.push(
            "Consider sharding across multiple ClickHouse nodes".to_string(),
        );
    }

    // Multi-tenant optimizations
    if workload.multi_tenant && workload.events_per_day >= TENANT_PROJECTION_THRESHOLD {
        recommendations.push(
            "Add a projection on `(project_id, toDate(timestamp))` for per-tenant dashboards"
                .to_string(),
        );
    }

    if workload.multi_tenant {
        recommendations.push(
            "Ensure `project_id` has `LowCardinality(String)` type for efficient filtering"
                .to_string(),
        );
    }

    // Always recommend compression guidance
    recommendations.push(
        "Use `LZ4` compression (ClickHouse default) — switch to `ZSTD` if storage-constrained"
            .to_string(),
    );

    recommendations
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::schema::{EventSchema, EventTable};

    fn make_workload(events_per_day: u64, multi_tenant: bool, retention_days: u32) -> (EventSchema, WorkloadProfile) {
        let schema = EventSchema {
            event_table: EventTable {
                name: "events".to_string(),
                multi_tenant,
                expected_events_per_day: events_per_day,
                retention_days,
            },
        };
        let workload = WorkloadProfile {
            events_per_day,
            multi_tenant,
            retention_days,
        };
        (schema, workload)
    }

    #[test]
    fn low_volume_single_tenant_with_retention() {
        let (schema, workload) = make_workload(1_000_000, false, 30);
        let recs = analyze(&schema, &workload);

        assert!(recs.iter().any(|r| r.contains("TTL") && r.contains("30 DAY")));
        assert!(recs.iter().any(|r| r.contains("LZ4")));
        assert!(!recs.iter().any(|r| r.contains("wide_parts_only")));
        assert!(!recs.iter().any(|r| r.contains("sharding")));
        assert!(!recs.iter().any(|r| r.contains("project_id")));
    }

    #[test]
    fn no_retention_warns_unbounded() {
        let (schema, workload) = make_workload(1_000_000, false, 0);
        let recs = analyze(&schema, &workload);

        assert!(recs.iter().any(|r| r.contains("unbounded")));
        assert!(!recs.iter().any(|r| r.contains("TTL timestamp")));
    }

    #[test]
    fn high_volume_enables_wide_parts() {
        let (schema, workload) = make_workload(HIGH_VOLUME_THRESHOLD, false, 90);
        let recs = analyze(&schema, &workload);

        assert!(recs.iter().any(|r| r.contains("wide_parts_only")));
        assert!(!recs.iter().any(|r| r.contains("sharding")));
    }

    #[test]
    fn very_high_volume_recommends_sharding() {
        let (schema, workload) = make_workload(SHARDING_THRESHOLD, false, 90);
        let recs = analyze(&schema, &workload);

        assert!(recs.iter().any(|r| r.contains("wide_parts_only")));
        assert!(recs.iter().any(|r| r.contains("sharding")));
    }

    #[test]
    fn multi_tenant_low_volume_gets_low_cardinality_only() {
        let (schema, workload) = make_workload(1_000_000, true, 90);
        let recs = analyze(&schema, &workload);

        assert!(recs.iter().any(|r| r.contains("LowCardinality")));
        assert!(!recs.iter().any(|r| r.contains("projection")));
    }

    #[test]
    fn multi_tenant_above_projection_threshold() {
        let (schema, workload) = make_workload(TENANT_PROJECTION_THRESHOLD, true, 90);
        let recs = analyze(&schema, &workload);

        assert!(recs.iter().any(|r| r.contains("projection") && r.contains("project_id")));
        assert!(recs.iter().any(|r| r.contains("LowCardinality")));
    }

    #[test]
    fn compression_always_recommended() {
        let (schema, workload) = make_workload(1, false, 0);
        let recs = analyze(&schema, &workload);

        assert!(recs.iter().any(|r| r.contains("LZ4") && r.contains("ZSTD")));
    }

    #[test]
    fn below_threshold_no_high_volume_recs() {
        let (schema, workload) = make_workload(HIGH_VOLUME_THRESHOLD - 1, false, 90);
        let recs = analyze(&schema, &workload);

        assert!(!recs.iter().any(|r| r.contains("wide_parts_only")));
        assert!(!recs.iter().any(|r| r.contains("sharding")));
    }

    #[test]
    fn multi_tenant_below_projection_threshold() {
        let (schema, workload) = make_workload(TENANT_PROJECTION_THRESHOLD - 1, true, 90);
        let recs = analyze(&schema, &workload);

        assert!(recs.iter().any(|r| r.contains("LowCardinality")));
        assert!(!recs.iter().any(|r| r.contains("projection")));
    }

    #[test]
    fn all_rules_fire_for_max_workload() {
        let (schema, workload) = make_workload(SHARDING_THRESHOLD, true, 365);
        let recs = analyze(&schema, &workload);

        assert!(recs.iter().any(|r| r.contains("TTL")));
        assert!(recs.iter().any(|r| r.contains("wide_parts_only")));
        assert!(recs.iter().any(|r| r.contains("sharding")));
        assert!(recs.iter().any(|r| r.contains("projection")));
        assert!(recs.iter().any(|r| r.contains("LowCardinality")));
        assert!(recs.iter().any(|r| r.contains("LZ4")));
        assert_eq!(recs.len(), 6);
    }
}
