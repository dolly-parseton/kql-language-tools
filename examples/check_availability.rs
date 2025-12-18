//! Library availability check
//!
//! Demonstrates checking if the native library is available before use.
//! Useful for applications that want graceful degradation.
//!
//! Run: `cargo run --example check_availability`

use kql_language_tools::{is_available, library_path, KqlValidator};

fn main() {
    println!("KQL Language Tools - Availability Check\n");

    // Check if native library is available
    if is_available() {
        println!("Native library: AVAILABLE");
        if let Some(path) = library_path() {
            println!("  Path: {}", path.display());
        }

        // Try to create validator
        match KqlValidator::new() {
            Ok(validator) => {
                println!("  Validator: initialized");

                // Check feature support
                println!("\nFeature support:");
                println!("  Schema validation:  {}", validator.supports_schema_validation());
                println!("  Completions:        {}", validator.supports_completion());
                println!("  Classifications:    {}", validator.supports_classification());
            }
            Err(e) => {
                println!("  Validator: failed to initialize");
                println!("  Error: {e}");
            }
        }
    } else {
        println!("Native library: NOT FOUND");
        println!();
        println!("To build the native library, ensure .NET 8+ SDK is installed and run:");
        println!("  cd dotnet && dotnet publish -c Release -r <your-platform>");
        println!();
        println!("Or set KQL_LANGUAGE_TOOLS_PATH to point to a pre-built library.");
    }
}
