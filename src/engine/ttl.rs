use crate::config::workload::WorkloadProfile;

pub fn suggest(workload: &WorkloadProfile) -> Option<String> {
    if workload.retention_days > 0 {
        Some(format!(
            "timestamp + INTERVAL {} DAY",
            workload.retention_days
        ))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(retention_days: u32) -> WorkloadProfile {
        WorkloadProfile {
            events_per_day: 1_000_000,
            multi_tenant: false,
            retention_days,
        }
    }

    #[test]
    fn suggests_ttl_for_positive_retention() {
        let result = suggest(&profile(90));
        assert_eq!(result, Some("timestamp + INTERVAL 90 DAY".to_string()));
    }

    #[test]
    fn suggests_ttl_for_one_day() {
        let result = suggest(&profile(1));
        assert_eq!(result, Some("timestamp + INTERVAL 1 DAY".to_string()));
    }

    #[test]
    fn returns_none_for_zero_retention() {
        let result = suggest(&profile(0));
        assert_eq!(result, None);
    }
}
