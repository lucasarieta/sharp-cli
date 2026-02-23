#[derive(Debug)]
pub struct CreateTable {
    pub table_name: String,
    pub columns: Vec<ColumnExpr>,
    pub engine: String,
    pub partition_by: Option<String>,
    pub order_by: Vec<String>,
    pub ttl: Option<String>,
}

#[derive(Debug)]
pub struct ColumnExpr {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
}

impl CreateTable {
    pub fn to_sql(&self) -> String {
        let mut sql = format!("CREATE TABLE {} (\n", self.table_name);

        let col_defs: Vec<String> = self
            .columns
            .iter()
            .map(|c| {
                let typ = if c.nullable {
                    format!("Nullable({})", c.data_type)
                } else {
                    c.data_type.clone()
                };
                format!("    {} {}", c.name, typ)
            })
            .collect();

        sql.push_str(&col_defs.join(",\n"));
        sql.push_str(&format!("\n) ENGINE = {}", self.engine));

        if let Some(ref p) = self.partition_by {
            sql.push_str(&format!("\nPARTITION BY {p}"));
        }

        if !self.order_by.is_empty() {
            sql.push_str(&format!("\nORDER BY ({})", self.order_by.join(", ")));
        }

        if let Some(ref t) = self.ttl {
            sql.push_str(&format!("\nTTL {t}"));
        }

        sql.push(';');
        sql
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_table() -> CreateTable {
        CreateTable {
            table_name: "events".to_string(),
            columns: vec![ColumnExpr {
                name: "id".to_string(),
                data_type: "UInt64".to_string(),
                nullable: false,
            }],
            engine: "MergeTree".to_string(),
            partition_by: None,
            order_by: vec![],
            ttl: None,
        }
    }

    #[test]
    fn minimal_table_sql() {
        let sql = minimal_table().to_sql();
        assert_eq!(
            sql,
            "CREATE TABLE events (\n    id UInt64\n) ENGINE = MergeTree;"
        );
    }

    #[test]
    fn nullable_column_wraps_type() {
        let table = CreateTable {
            columns: vec![ColumnExpr {
                name: "email".to_string(),
                data_type: "String".to_string(),
                nullable: true,
            }],
            ..minimal_table()
        };
        let sql = table.to_sql();
        assert!(sql.contains("email Nullable(String)"));
    }

    #[test]
    fn multiple_columns_comma_separated() {
        let table = CreateTable {
            columns: vec![
                ColumnExpr {
                    name: "a".to_string(),
                    data_type: "UInt32".to_string(),
                    nullable: false,
                },
                ColumnExpr {
                    name: "b".to_string(),
                    data_type: "String".to_string(),
                    nullable: false,
                },
            ],
            ..minimal_table()
        };
        let sql = table.to_sql();
        assert!(sql.contains("    a UInt32,\n    b String"));
    }

    #[test]
    fn partition_by_rendered_when_present() {
        let table = CreateTable {
            partition_by: Some("toYYYYMM(ts)".to_string()),
            ..minimal_table()
        };
        let sql = table.to_sql();
        assert!(sql.contains("PARTITION BY toYYYYMM(ts)"));
    }

    #[test]
    fn partition_by_omitted_when_none() {
        let sql = minimal_table().to_sql();
        assert!(!sql.contains("PARTITION BY"));
    }

    #[test]
    fn order_by_rendered_when_present() {
        let table = CreateTable {
            order_by: vec!["a".to_string(), "b".to_string()],
            ..minimal_table()
        };
        let sql = table.to_sql();
        assert!(sql.contains("ORDER BY (a, b)"));
    }

    #[test]
    fn order_by_omitted_when_empty() {
        let sql = minimal_table().to_sql();
        assert!(!sql.contains("ORDER BY"));
    }

    #[test]
    fn ttl_rendered_when_present() {
        let table = CreateTable {
            ttl: Some("ts + INTERVAL 30 DAY".to_string()),
            ..minimal_table()
        };
        let sql = table.to_sql();
        assert!(sql.contains("TTL ts + INTERVAL 30 DAY"));
    }

    #[test]
    fn ttl_omitted_when_none() {
        let sql = minimal_table().to_sql();
        assert!(!sql.contains("TTL"));
    }

    #[test]
    fn full_statement_clauses_in_order() {
        let table = CreateTable {
            table_name: "t".to_string(),
            columns: vec![ColumnExpr {
                name: "id".to_string(),
                data_type: "UInt64".to_string(),
                nullable: false,
            }],
            engine: "MergeTree".to_string(),
            partition_by: Some("toYYYYMMDD(ts)".to_string()),
            order_by: vec!["id".to_string()],
            ttl: Some("ts + INTERVAL 7 DAY".to_string()),
        };
        let sql = table.to_sql();

        let engine_pos = sql.find("ENGINE").unwrap();
        let partition_pos = sql.find("PARTITION BY").unwrap();
        let order_pos = sql.find("ORDER BY").unwrap();
        let ttl_pos = sql.find("TTL").unwrap();

        assert!(engine_pos < partition_pos);
        assert!(partition_pos < order_pos);
        assert!(order_pos < ttl_pos);
        assert!(sql.ends_with(';'));
    }
}
