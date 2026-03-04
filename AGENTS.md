# excalidraw-rs

Rust CLI tool for editing Excalidraw diagrams. Drop-in replacement for the TypeScript version with ~75x faster startup and batch mode support.

## Build

```bash
cargo build --release
```

Binary: `target/release/excalidraw-rs`

## Architecture

```
src/
├── main.rs     # CLI (clap derive)
├── model.rs    # ExcalidrawFile, Element, color presets
├── io.rs       # Load/save .excalidraw and .excalidraw.md (LZ-String)
├── ops.rs      # create_node, create_edge, delete_element
├── query.rs    # find_by_id, find_by_label, resolve_node, auto-position
├── format.rs   # parse_diagram, format_diagram_markdown, emoji mapping
└── batch.rs    # Batch DSL parser + executor
```

## Key Design Decisions

- **`#[serde(flatten)]` for passthrough.** Element has ~20 typed fields; everything else goes through `extra: Map<String, Value>` untouched. Safe against Excalidraw format changes.
- **Index-based references.** `query::resolve_node` returns an index, not a reference — avoids borrow checker issues when mutating elements.
- **Batch errors are collected, not fatal.** One bad line doesn't stop the rest from executing.

## Tests

```bash
cargo test
```

Integration tests in `tests/integration.rs` exercise all CLI subcommands against temp files.
