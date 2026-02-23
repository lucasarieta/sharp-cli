use crate::sql::ast::CreateTable;

pub fn print_sql(stmt: &CreateTable) {
    println!("{}", stmt.to_sql());
}
