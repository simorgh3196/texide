# Texide JSON Schemas

This directory contains JSON Schema definitions for Texide rule development.

## Files

- `rule-types.json` - All type definitions for WASM rules

## Usage

### Rust

Use the `texide-rule-common` crate which implements these types:

```rust
use texide_rule_common::{
    LintRequest, LintResponse, Diagnostic, Span, Fix, RuleManifest
};
```

### TypeScript / AssemblyScript

Generate types using [quicktype](https://quicktype.io/):

```bash
# Install quicktype
npm install -g quicktype

# Generate TypeScript types
quicktype schemas/rule-types.json \
  --src-lang schema \
  --lang typescript \
  --out src/types.ts

# Generate AssemblyScript (experimental)
quicktype schemas/rule-types.json \
  --src-lang schema \
  --lang typescript \
  --out src/types.ts
# Then manually adapt for AssemblyScript
```

### Go

```bash
# Using gojsonschema
go install github.com/atombender/go-jsonschema/cmd/gojsonschema@latest

gojsonschema -p types schemas/rule-types.json -o types/rule_types.go
```

### Other Languages

Use any JSON Schema code generator for your target language:
- Python: `datamodel-code-generator`
- Java: `jsonschema2pojo`
- C#: `NJsonSchema`

## Type Reference

### RuleManifest

Returned by `get_manifest()` function.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| name | string | Yes | Rule identifier (e.g., "no-todo") |
| version | string | Yes | Semver version (e.g., "1.0.0") |
| description | string | No | Human-readable description |
| fixable | boolean | No | Whether rule provides auto-fixes |
| node_types | string[] | No | Node types to receive (empty = all) |
| schema | object | No | JSON Schema for config options |

### LintRequest

Input to `lint()` function.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| node | AstNode | Yes | AST node to lint |
| config | object | Yes | Rule configuration |
| source | string | Yes | Full source text |
| file_path | string? | No | File path (if available) |

### LintResponse

Output from `lint()` function.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| diagnostics | Diagnostic[] | Yes | Array of lint results |

### Diagnostic

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| rule_id | string | Yes | Rule identifier |
| message | string | Yes | Error message |
| span | Span | Yes | Source location |
| severity | Severity | No | "error" / "warning" / "info" |
| fix | Fix | No | Auto-fix (if fixable) |

### Span

| Field | Type | Description |
|-------|------|-------------|
| start | u32 | Start byte offset (inclusive) |
| end | u32 | End byte offset (exclusive) |

### Fix

| Field | Type | Description |
|-------|------|-------------|
| span | Span | Range to replace |
| text | string | Replacement text |

## Validation

Validate your rule output:

```bash
# Using ajv-cli
npm install -g ajv-cli

# Validate a manifest
ajv validate -s schemas/rule-types.json \
  --spec=draft7 \
  -r '#/$defs/RuleManifest' \
  -d manifest.json
```
