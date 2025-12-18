//! Syntax highlighting with classifications
//!
//! Demonstrates getting classification spans for syntax highlighting.
//!
//! Run: `cargo run --example syntax_highlighting`

use kql_language_tools::{KqlValidator, Error, ClassificationKind};

fn main() -> Result<(), Error> {
    let validator = KqlValidator::new()?;

    let query = "SecurityEvents | where TimeGenerated > ago(1h) | take 10";
    println!("Query: {query}\n");

    let result = validator.get_classifications(query)?;

    println!("Classifications:");
    for span in &result.spans {
        let text = &query[span.start..span.start + span.length];
        println!("  {:20} {:?} ({}..{})",
            format!("\"{text}\""), span.kind, span.start, span.start + span.length);
    }

    // Demonstrate colorized output
    println!("\nColorized (ANSI):");
    print_colorized(query, &result.spans);

    Ok(())
}

fn kind_to_color(kind: ClassificationKind) -> &'static str {
    match kind {
        ClassificationKind::Keyword | ClassificationKind::QueryOperator => "\x1b[94m",   // Blue
        ClassificationKind::ScalarFunction | ClassificationKind::AggregateFunction => "\x1b[93m", // Yellow
        ClassificationKind::StringLiteral => "\x1b[92m",  // Green
        ClassificationKind::Literal => "\x1b[95m",        // Magenta
        ClassificationKind::Comment => "\x1b[90m",        // Gray
        ClassificationKind::Table => "\x1b[96m",          // Cyan
        ClassificationKind::Column => "\x1b[97m",         // White
        _ => "\x1b[0m",                                   // Reset
    }
}

fn print_colorized(query: &str, spans: &[kql_language_tools::ClassifiedSpan]) {
    let mut last_end = 0;

    for span in spans {
        // Print any gap as plain text
        if span.start > last_end {
            print!("{}", &query[last_end..span.start]);
        }

        // Print colored span
        let text = &query[span.start..span.start + span.length];
        let color = kind_to_color(span.kind);
        print!("{color}{text}\x1b[0m");

        last_end = span.start + span.length;
    }

    // Print remainder
    if last_end < query.len() {
        print!("{}", &query[last_end..]);
    }
    println!();
}
