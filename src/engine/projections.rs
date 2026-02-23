pub fn daily_event_projection(table_name: &str) -> String {
    format!(
        "\
ALTER TABLE {table_name}
ADD PROJECTION daily_event_counts
(
    SELECT
        project_id,
        toDate(timestamp) AS day,
        event_name,
        count()
    GROUP BY project_id, day, event_name
);"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_projection_sql() {
        let sql = daily_event_projection("user_events");
        assert!(sql.starts_with("ALTER TABLE user_events"));
        assert!(sql.contains("ADD PROJECTION daily_event_counts"));
        assert!(sql.contains("toDate(timestamp) AS day"));
        assert!(sql.contains("count()"));
        assert!(sql.contains("GROUP BY project_id, day, event_name"));
    }

    #[test]
    fn uses_table_name() {
        let sql = daily_event_projection("custom_events");
        assert!(sql.starts_with("ALTER TABLE custom_events"));
    }
}
