use crate::config::workload::WorkloadProfile;

#[derive(Debug, PartialEq)]
pub enum PartitionStrategy {
    Monthly,
    Daily,
    DailyWithTenant,
}

impl PartitionStrategy {
    pub fn to_sql(&self) -> String {
        match self {
            Self::Monthly => "toYYYYMM(timestamp)".to_string(),
            Self::Daily => "toYYYYMMDD(timestamp)".to_string(),
            Self::DailyWithTenant => {
                "tuple(project_id, toYYYYMMDD(timestamp))".to_string()
            }
        }
    }

    pub fn explain(&self, workload: &WorkloadProfile) -> String {
        match self {
            Self::Monthly => format!(
                "Monthly partitioning selected: {} events/day is under the 5M threshold, \
                 so monthly granularity avoids excessive part count.",
                workload.events_per_day
            ),
            Self::Daily => format!(
                "Daily partitioning selected: {} events/day falls in the 5M–200M range, \
                 balancing query pruning against part management overhead.",
                workload.events_per_day
            ),
            Self::DailyWithTenant => format!(
                "Daily + tenant partitioning selected: {} events/day exceeds 200M \
                 with multi-tenant enabled, partitioning by (project_id, day) \
                 isolates tenant data for efficient pruning.",
                workload.events_per_day
            ),
        }
    }
}

pub fn choose_partition_strategy(workload: &WorkloadProfile) -> PartitionStrategy {
    if workload.events_per_day < 5_000_000 {
        PartitionStrategy::Monthly
    } else if workload.events_per_day <= 200_000_000 {
        PartitionStrategy::Daily
    } else if workload.multi_tenant {
        PartitionStrategy::DailyWithTenant
    } else {
        PartitionStrategy::Daily
    }
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
    fn low_volume_uses_monthly() {
        let w = profile(1_000_000, false);
        assert_eq!(choose_partition_strategy(&w), PartitionStrategy::Monthly);
    }

    #[test]
    fn medium_volume_uses_daily() {
        let w = profile(50_000_000, false);
        assert_eq!(choose_partition_strategy(&w), PartitionStrategy::Daily);
    }

    #[test]
    fn boundary_5m_uses_daily() {
        let w = profile(5_000_000, false);
        assert_eq!(choose_partition_strategy(&w), PartitionStrategy::Daily);
    }

    #[test]
    fn boundary_200m_uses_daily() {
        let w = profile(200_000_000, false);
        assert_eq!(choose_partition_strategy(&w), PartitionStrategy::Daily);
    }

    #[test]
    fn high_volume_multi_tenant_uses_daily_with_tenant() {
        let w = profile(500_000_000, true);
        assert_eq!(
            choose_partition_strategy(&w),
            PartitionStrategy::DailyWithTenant
        );
    }

    #[test]
    fn high_volume_single_tenant_falls_back_to_daily() {
        let w = profile(500_000_000, false);
        assert_eq!(choose_partition_strategy(&w), PartitionStrategy::Daily);
    }

    #[test]
    fn monthly_sql() {
        assert_eq!(PartitionStrategy::Monthly.to_sql(), "toYYYYMM(timestamp)");
    }

    #[test]
    fn daily_sql() {
        assert_eq!(PartitionStrategy::Daily.to_sql(), "toYYYYMMDD(timestamp)");
    }

    #[test]
    fn daily_with_tenant_sql() {
        assert_eq!(
            PartitionStrategy::DailyWithTenant.to_sql(),
            "tuple(project_id, toYYYYMMDD(timestamp))"
        );
    }
}
