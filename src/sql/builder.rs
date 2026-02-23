use crate::config::schema::EventSchema;
use crate::sql::ast::{ColumnExpr, CreateTable};

pub fn build_create_table_sql(
    schema: &EventSchema,
    partition_sql: &str,
    order_by_cols: &[String],
    ttl: Option<String>,
) -> CreateTable {
    let table = &schema.event_table;

    let columns = vec![
        ColumnExpr {
            name: "project_id".to_string(),
            data_type: "UInt32".to_string(),
            nullable: false,
        },
        ColumnExpr {
            name: "timestamp".to_string(),
            data_type: "DateTime".to_string(),
            nullable: false,
        },
        ColumnExpr {
            name: "event_name".to_string(),
            data_type: "LowCardinality(String)".to_string(),
            nullable: false,
        },
        ColumnExpr {
            name: "distinct_id".to_string(),
            data_type: "String".to_string(),
            nullable: false,
        },
        ColumnExpr {
            name: "properties".to_string(),
            data_type: "JSON".to_string(),
            nullable: false,
        },
    ];

    CreateTable {
        table_name: table.name.clone(),
        columns,
        engine: "MergeTree".to_string(),
        partition_by: Some(partition_sql.to_string()),
        order_by: order_by_cols.to_vec(),
        ttl,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::schema::{EventSchema, EventTable};

    fn test_schema() -> EventSchema {
        EventSchema {
            event_table: EventTable {
                name: "analytics_events".to_string(),
                multi_tenant: true,
                expected_events_per_day: 50_000_000,
                retention_days: 90,
            },
        }
    }

    #[test]
    fn generates_full_create_table() {
        let schema = test_schema();
        let order_cols = vec![
            "project_id".to_string(),
            "event_name".to_string(),
            "timestamp".to_string(),
            "distinct_id".to_string(),
        ];
        let ast = build_create_table_sql(
            &schema,
            "toYYYYMMDD(timestamp)",
            &order_cols,
            Some("timestamp + INTERVAL 90 DAY".to_string()),
        );
        let sql = ast.to_sql();

        assert!(sql.starts_with("CREATE TABLE analytics_events ("));
        assert!(sql.contains("project_id UInt32"));
        assert!(sql.contains("timestamp DateTime"));
        assert!(sql.contains("event_name LowCardinality(String)"));
        assert!(sql.contains("distinct_id String"));
        assert!(sql.contains("properties JSON"));
        assert!(sql.contains("ENGINE = MergeTree"));
        assert!(sql.contains("PARTITION BY toYYYYMMDD(timestamp)"));
        assert!(sql.contains("ORDER BY (project_id, event_name, timestamp, distinct_id)"));
        assert!(sql.contains("TTL timestamp + INTERVAL 90 DAY"));
    }

    #[test]
    fn uses_schema_name_and_retention() {
        let mut schema = test_schema();
        schema.event_table.name = "custom_events".to_string();
        schema.event_table.retention_days = 30;

        let ast = build_create_table_sql(
            &schema,
            "toYYYYMM(timestamp)",
            &["timestamp".to_string()],
            Some("timestamp + INTERVAL 30 DAY".to_string()),
        );
        let sql = ast.to_sql();

        assert!(sql.starts_with("CREATE TABLE custom_events ("));
        assert!(sql.contains("INTERVAL 30 DAY"));
    }

    #[test]
    fn no_ttl_when_none() {
        let schema = test_schema();
        let ast = build_create_table_sql(
            &schema,
            "toYYYYMMDD(timestamp)",
            &["timestamp".to_string()],
            None,
        );
        let sql = ast.to_sql();
        assert!(!sql.contains("TTL"));
    }

    #[test]
    fn ast_has_correct_column_count() {
        let schema = test_schema();
        let ast = build_create_table_sql(
            &schema,
            "toYYYYMMDD(timestamp)",
            &["timestamp".to_string()],
            None,
        );
        assert_eq!(ast.columns.len(), 5);
        assert_eq!(ast.columns[0].name, "project_id");
        assert_eq!(ast.columns[4].name, "properties");
    }

    #[test]
    fn ast_engine_is_mergetree() {
        let schema = test_schema();
        let ast = build_create_table_sql(
            &schema,
            "toYYYYMMDD(timestamp)",
            &["timestamp".to_string()],
            None,
        );
        assert_eq!(ast.engine, "MergeTree");
    }
}
