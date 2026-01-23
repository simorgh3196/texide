# Texide Sample Rules

This directory contains sample WASM rules for Texide.

## Available Rules

| Rule | Description |
|------|-------------|
| `no-todo` | Disallow TODO/FIXME comments |
| `sentence-length` | Check sentence length |
| `no-doubled-joshi` | Detect repeated Japanese particles (助詞) |

## Building Rules

### Prerequisites

```bash
# Install WASM target
rustup target add wasm32-wasip1
```

### Build All Rules

```bash
cd rules
cargo build --target wasm32-wasip1 --release
```

### Build Specific Rule

```bash
cargo build --target wasm32-wasip1 --release -p texide-rule-no-todo
```

### Output

Built WASM files are located at:
```
rules/target/wasm32-wasip1/release/
├── texide_rule_no_todo.wasm
├── texide_rule_sentence_length.wasm
└── texide_rule_no_doubled_joshi.wasm
```

## Testing

### Unit Tests

```bash
cd rules
cargo test --workspace
```

## Rule Configuration

### no-todo

Detects TODO/FIXME/XXX comments in text.

```json
{
  "rules": {
    "no-todo": {
      "patterns": ["TODO:", "FIXME:", "HACK:"],
      "ignore_patterns": ["TODO-OK:"],
      "case_sensitive": false
    }
  }
}
```

### sentence-length

Checks sentence length.

```json
{
  "rules": {
    "sentence-length": {
      "max": 100,
      "skip_code": true
    }
  }
}
```

### no-doubled-joshi

Detects repeated Japanese particles.

```json
{
  "rules": {
    "no-doubled-joshi": {
      "particles": ["は", "が", "を", "に", "で", "と", "も", "の"],
      "min_interval": 0,
      "allow": [],
      "suggest_fix": true
    }
  }
}
```

## Developing New Rules

See [Rule Development Guide](../docs/rule-development.md) for detailed instructions.

### Rule Interface

Rules must implement two functions:

1. `get_manifest()` - Returns rule metadata
2. `lint(input: String)` - Performs linting

### Using Common Types

```rust
use texide_rule_common::{
    Diagnostic, Fix, LintRequest, LintResponse, RuleManifest, Span,
    extract_node_text, is_node_type,
};
```
