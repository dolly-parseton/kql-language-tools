//! Code completion / intellisense
//!
//! Demonstrates getting completion suggestions at cursor positions.
//!
//! Run: cargo run --example completions

use kql_language_tools::{CompletionKind, Error, KqlValidator, Schema, Table};

fn main() -> Result<(), Error> {
    let validator = KqlValidator::new()?;

    // Schema for context-aware completions
    let schema = Schema::new().table(
        Table::new("Events")
            .with_column("Timestamp", "datetime")
            .with_column("Message", "string")
            .with_column("Level", "long"),
    );

    // Completions after pipe - show operators
    let query = "Events | ";
    let cursor = query.len();
    println!("Query: \"{query}\" (cursor at {cursor})");

    let result = validator.get_completions(query, cursor, Some(&schema))?;
    println!("Suggestions ({} items):", result.items.len());
    for item in result.items.iter().take(10) {
        println!("  {:20} {:?}", item.label, item.kind);
    }

    // Completions for column names after 'where'
    let query = "Events | where ";
    let cursor = query.len();
    println!("\nQuery: \"{query}\" (cursor at {cursor})");

    let result = validator.get_completions(query, cursor, Some(&schema))?;
    println!("Column suggestions:");
    for item in result
        .items
        .iter()
        .filter(|i| i.kind == CompletionKind::Column)
    {
        println!("  {} - {:?}", item.label, item.detail);
    }

    // Completions for functions
    let query = "Events | where Timestamp > ";
    let cursor = query.len();
    println!("\nQuery: \"{query}\" (cursor at {cursor})");

    let result = validator.get_completions(query, cursor, Some(&schema))?;
    println!("Function suggestions:");
    for item in result
        .items
        .iter()
        .filter(|i| {
            matches!(
                i.kind,
                CompletionKind::Function | CompletionKind::AggregateFunction
            )
        })
        .take(10)
    {
        println!("  {}", item.label);
    }

    Ok(())
}
