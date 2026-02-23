mod cli;
mod config;
mod engine;
mod errors;
mod output;
mod sql;

use clap::Parser;
use cli::{Cli, Command};
use config::workload::WorkloadProfile;
use engine::ordering;
use engine::partitioning::choose_partition_strategy;
use sql::builder::build_create_table_sql;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Init => {
            println!("Initialized default schema config.");
        }
        Command::Generate { schema } => {
            let event_schema = match config::load_schema(&schema) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            };
            let workload = WorkloadProfile::from_schema(&event_schema);
            let partition = choose_partition_strategy(&workload);
            let order_cols = ordering::choose_order_by(&workload);
            let ttl = engine::ttl::suggest(&workload);

            let ast = build_create_table_sql(
                &event_schema,
                &partition.to_sql(),
                &order_cols,
                ttl,
            );
            output::formatter::print_sql(&ast);
            println!();
            let projection_sql =
                engine::projections::daily_event_projection(&event_schema.event_table.name);
            println!("{projection_sql}");
        }
        Command::Explain { schema } => {
            let event_schema = match config::load_schema(&schema) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            };
            let workload = WorkloadProfile::from_schema(&event_schema);
            let partition = choose_partition_strategy(&workload);
            let order_cols = ordering::choose_order_by(&workload);

            println!("Partitioning:");
            println!("  Strategy: {partition:?}");
            println!("  SQL:      PARTITION BY {}", partition.to_sql());
            println!("  Reason:   {}", partition.explain(&workload));
            println!();
            println!("Ordering:");
            println!("  SQL:      {}", ordering::order_by_sql(&order_cols));
            println!(
                "  Reason:   {}",
                ordering::explain(&workload, &order_cols)
            );
            println!();

            let recommendations = engine::heuristics::analyze(&event_schema, &workload);
            println!("Recommendations:");
            for rec in &recommendations {
                println!("  - {rec}");
            }
        }
    }
}
