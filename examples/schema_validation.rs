//! Schema-aware KQL validation
//!
//! Demonstrates validating queries against a defined schema with tables and columns.
//!
//! Run: `cargo run --example schema_validation`

use kql_language_tools::{KqlValidator, Schema, Table, Error};

fn main() -> Result<(), Error> {
    let validator = KqlValidator::new()?;

    // Define a schema with tables and columns
    let schema = Schema::new()
        .table(
            Table::new("SecurityEvents")
                .with_column("TimeGenerated", "datetime")
                .with_column("EventID", "long")
                .with_column("Computer", "string")
                .with_column("Account", "string")
        )
        .table(
            Table::new("SigninLogs")
                .with_column("TimeGenerated", "datetime")
                .with_column("UserPrincipalName", "string")
                .with_column("IPAddress", "string")
                .with_column("ResultType", "string")
        );

    // Valid query - table and columns exist
    let query = "SecurityEvents | where EventID == 4624 | project TimeGenerated, Account";
    let result = validator.validate_with_schema(query, &schema)?;
    println!("Query against known schema: valid={}", result.is_valid());

    // Invalid - unknown table
    let query = "UnknownTable | take 10";
    let result = validator.validate_with_schema(query, &schema)?;
    println!("\nUnknown table:");
    for diag in result.diagnostics() {
        println!("  {}", diag.message);
    }

    // Invalid - unknown column
    let query = "SecurityEvents | where NonExistentColumn == 'x'";
    let result = validator.validate_with_schema(query, &schema)?;
    println!("\nUnknown column:");
    for diag in result.diagnostics() {
        println!("  {}", diag.message);
    }

    // Type mismatch (comparing string to number)
    let query = "SecurityEvents | where Computer == 123";
    let result = validator.validate_with_schema(query, &schema)?;
    println!("\nType mismatch:");
    for diag in result.diagnostics() {
        println!("  {}", diag.message);
    }

    Ok(())
}
