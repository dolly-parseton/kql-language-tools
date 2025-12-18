# Future Development

This document outlines planned features and improvements for kql-language-tools.

## Planned Features

### Data Lineage

Extract tables and columns referenced by a query.

```rust
let refs = validator.get_references(query, &schema)?;

for table in &refs.tables {
    println!("Table: {}", table);
}

for column in &refs.columns {
    println!("Column: {}.{}", column.table, column.name);
}
```

**Use cases:**
- Query dependency analysis
- Schema usage tracking
- Data access auditing

**FFI addition:** `kql_get_references`

---

### Symbol Resolution

Resolve what a name refers to at a given position.

```rust
let symbol = validator.get_symbol_at(query, position, &schema)?;

match symbol.kind {
    SymbolKind::Column => println!("Column {} from {}", symbol.name, symbol.table),
    SymbolKind::Table => println!("Table {}", symbol.name),
    SymbolKind::Function => println!("Function {}", symbol.name),
    _ => {}
}
```

**Use cases:**
- Go-to-definition
- Find references
- Hover information

**FFI addition:** `kql_get_symbol_at`

---

### Type Information

Get the result type of an expression at a given position.

```rust
let type_info = validator.get_type_at(query, position, &schema)?;

println!("Type: {}", type_info.name);       // "datetime", "string", etc.
println!("Scalar: {}", type_info.is_scalar); // true for columns
println!("Tabular: {}", type_info.is_table); // true for table expressions
```

**Use cases:**
- Type-aware completions
- Expression validation
- Documentation generation

**FFI addition:** `kql_get_type_at`

---

### AST Access

Expose the parsed syntax tree for query analysis and transformation.

Two potential approaches under consideration:

**JSON-serialized AST:**
```rust
let ast = validator.get_ast(query)?;

fn visit(node: &AstNode) {
    println!("{:?} at {}..{}", node.kind, node.start, node.end);
    for child in &node.children {
        visit(child);
    }
}
visit(&ast.root);
```

**Handle-based API:**
```rust
let handle = validator.parse(query)?;
let root = handle.root_node();

for child in root.children() {
    println!("{:?}", child.kind());
}

// Explicit cleanup
drop(handle);
```

**Use cases:**
- Query rewriting
- Static analysis tools
- Custom linting rules

**FFI additions:** Either `kql_get_ast` or `kql_parse`/`kql_get_node_at`/`kql_release_handle`

---

### Function Body Resolution

Retrieve the body of a user-defined function referenced in a query.

```rust
let func = validator.get_function_body("GetRecentEvents", &schema)?;

println!("Body: {}", func.body);
println!("Parameters: {:?}", func.parameters);
```

**Use cases:**
- Function inlining
- Dependency analysis across functions

**FFI addition:** `kql_get_function_body`

---

## Infrastructure Improvements

### Native Library Distribution

- Pre-built binaries for common platforms (GitHub releases)
- `bundled` feature flag for embedding binaries in crate
- Automatic download during build when native library missing

### Platform Verification

- CI testing across all supported platforms
- Integration tests for each FFI function
- Performance benchmarks for large queries

### Error Handling

- Structured error codes aligned with Kusto.Language
- Error recovery suggestions where applicable
- Clearer messages for common mistakes

---

## API Stability

Current API (`validate_syntax`, `validate_with_schema`, `get_completions`, `get_classifications`) is stable and unlikely to change.

New features will be added as separate methods without breaking existing consumers.

---

## Contributing

Feature requests and contributions welcome. For new FFI functions:

1. Add export in `dotnet/src/NativeExports.cs`
2. Add Rust types in `src/types.rs`
3. Add safe wrapper in `src/validator.rs`
4. Add tests covering success and error cases