//! Basic KQL syntax validation
//!
//! Demonstrates checking queries for syntax errors without schema awareness.
//!
//! Run: `cargo run --example basic_validation`

use kql_language_tools::{KqlValidator, Error};

fn main() -> Result<(), Error> {
    let validator = KqlValidator::new()?;

    // Valid query
    let result = validator.validate_syntax("StormEvents | where State == 'TEXAS' | take 10")?;
    println!("Valid query: {}", result.is_valid());

    // Invalid query - syntax error
    let result = validator.validate_syntax("StormEvents | where")?;
    println!("\nInvalid query diagnostics:");
    for diag in result.diagnostics() {
        println!("  [{:?}] {} (line {}, col {})",
            diag.severity, diag.message, diag.line, diag.column);
    }

    // Multiple errors
    let result = validator.validate_syntax("| where x == | take")?;
    println!("\nMultiple errors ({} total):", result.diagnostics().len());
    for diag in result.errors() {
        println!("  {}", diag.message);
    }

    Ok(())
}
