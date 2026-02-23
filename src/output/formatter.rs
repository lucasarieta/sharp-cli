use crate::sql::ast::CreateTable;

pub fn print_sql(stmt: &CreateTable) {
    println!("{}", stmt.to_sql());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::ast::ColumnExpr;

    #[test]
    fn print_sql_does_not_panic() {
        let stmt = CreateTable {
            table_name: "test".to_string(),
            columns: vec![ColumnExpr {
                name: "id".to_string(),
                data_type: "UInt64".to_string(),
                nullable: false,
            }],
            engine: "MergeTree".to_string(),
            partition_by: None,
            order_by: vec![],
            ttl: None,
        };
        // Verify print_sql completes without panicking
        print_sql(&stmt);
    }
}
