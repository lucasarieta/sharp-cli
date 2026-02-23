use crate::config::workload::WorkloadProfile;

const HIGH_VOLUME_THRESHOLD: u64 = 100_000_000;

pub fn choose_order_by(workload: &WorkloadProfile) -> Vec<String> {
    let mut cols: Vec<String> = if workload.multi_tenant {
        ["project_id", "event_name", "timestamp", "distinct_id"]
            .into_iter()
            .map(String::from)
            .collect()
    } else {
        ["event_name", "timestamp", "distinct_id"]
            .into_iter()
            .map(String::from)
            .collect()
    };

    if workload.events_per_day > HIGH_VOLUME_THRESHOLD {
        promote_timestamp(&mut cols);
    }

    cols
}

fn promote_timestamp(cols: &mut Vec<String>) {
    if let Some(pos) = cols.iter().position(|c| c == "timestamp") {
        cols.remove(pos);
        let insert_at = cols
            .iter()
            .position(|c| c != "project_id")
            .unwrap_or(0);
        cols.insert(insert_at, "timestamp".to_string());
    }
}

pub fn order_by_sql(cols: &[String]) -> String {
    format!("ORDER BY ({})", cols.join(", "))
}

pub fn explain(workload: &WorkloadProfile, cols: &[String]) -> String {
    let base = if workload.multi_tenant {
        "Multi-tenant: project_id leads the key for tenant isolation."
    } else {
        "Single-tenant: no project_id prefix needed."
    };

    let volume = if workload.events_per_day > HIGH_VOLUME_THRESHOLD {
        " High volume (>100M events/day): timestamp promoted earlier for time-range pruning."
    } else {
        " Standard volume: event_name before timestamp optimizes per-event-type queries."
    };

    format!("{base}{volume} Final key: ({}).", cols.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(events_per_day: u64, multi_tenant: bool) -> WorkloadProfile {
        WorkloadProfile {
            events_per_day,
            multi_tenant,
            retention_days: 90,
        }
    }

    #[test]
    fn single_tenant_default() {
        let cols = choose_order_by(&profile(1_000_000, false));
        assert_eq!(cols, ["event_name", "timestamp", "distinct_id"]);
    }

    #[test]
    fn multi_tenant_default() {
        let cols = choose_order_by(&profile(1_000_000, true));
        assert_eq!(
            cols,
            ["project_id", "event_name", "timestamp", "distinct_id"]
        );
    }

    #[test]
    fn high_volume_single_tenant_promotes_timestamp() {
        let cols = choose_order_by(&profile(200_000_000, false));
        assert_eq!(cols, ["timestamp", "event_name", "distinct_id"]);
    }

    #[test]
    fn high_volume_multi_tenant_promotes_timestamp_after_project_id() {
        let cols = choose_order_by(&profile(200_000_000, true));
        assert_eq!(
            cols,
            ["project_id", "timestamp", "event_name", "distinct_id"]
        );
    }

    #[test]
    fn boundary_100m_no_promotion() {
        let cols = choose_order_by(&profile(100_000_000, false));
        assert_eq!(cols, ["event_name", "timestamp", "distinct_id"]);
    }

    #[test]
    fn boundary_just_above_100m_promotes() {
        let cols = choose_order_by(&profile(100_000_001, false));
        assert_eq!(cols, ["timestamp", "event_name", "distinct_id"]);
    }

    #[test]
    fn order_by_sql_format() {
        let cols: Vec<String> = ["a", "b", "c"].into_iter().map(String::from).collect();
        assert_eq!(order_by_sql(&cols), "ORDER BY (a, b, c)");
    }
}
