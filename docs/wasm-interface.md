# WASM Rule Interface Specification

This document defines the language-agnostic interface that all Texide WASM rules must implement.

## Overview

Texide rules are compiled to WebAssembly (WASM) and run in a sandboxed environment. This specification enables rule development in any language that compiles to WASM (Rust, AssemblyScript, Go, etc.).

## Target

Rules must compile to `wasm32-wasip1` (WASI Preview 1).

## Required Exports

Every rule WASM module must export these two functions:

### `get_manifest`

Returns metadata about the rule.

```
Signature: () -> i32 (pointer to JSON string)
```

**Response**: JSON string matching [RuleManifest schema](#rulemanifest)

### `lint`

Performs linting on a single AST node.

```
Signature: (input_ptr: i32, input_len: i32) -> i32 (pointer to JSON string)
```

**Input**: JSON string matching [LintRequest schema](#lintrequest)
**Response**: JSON string matching [LintResponse schema](#lintresponse)

## Memory Management

### For Extism-based Runtimes (Recommended)

When using Extism PDK, memory management is handled automatically:

- **Rust**: Use `extism-pdk` crate with `#[plugin_fn]` macro
- **AssemblyScript**: Use `@aspect/as-pdk` package
- **Go**: Use `github.com/extism/go-pdk` package

### For Custom Runtimes

If implementing without Extism PDK, export these functions:

```
alloc(size: i32) -> i32    // Allocate memory, return pointer
dealloc(ptr: i32, size: i32)  // Free memory (optional)
```

## Data Schemas

### RuleManifest

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["name", "version"],
  "properties": {
    "name": {
      "type": "string",
      "description": "Unique rule identifier (e.g., 'no-todo')",
      "pattern": "^[a-z][a-z0-9-]*$"
    },
    "version": {
      "type": "string",
      "description": "Semantic version (e.g., '1.0.0')",
      "pattern": "^\\d+\\.\\d+\\.\\d+(-[a-zA-Z0-9.]+)?$"
    },
    "description": {
      "type": "string",
      "description": "Human-readable description"
    },
    "fixable": {
      "type": "boolean",
      "default": false,
      "description": "Whether this rule provides auto-fixes"
    },
    "node_types": {
      "type": "array",
      "items": { "type": "string" },
      "default": [],
      "description": "Node types to receive (empty = all nodes)"
    },
    "schema": {
      "type": "object",
      "description": "JSON Schema for rule configuration options"
    }
  }
}
```

### LintRequest

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["node", "config", "source"],
  "properties": {
    "node": {
      "type": "object",
      "description": "AST node to lint",
      "properties": {
        "type": { "type": "string" },
        "range": {
          "type": "array",
          "items": { "type": "integer" },
          "minItems": 2,
          "maxItems": 2,
          "description": "[start, end] byte offsets"
        },
        "children": {
          "type": "array",
          "items": { "$ref": "#/properties/node" }
        }
      }
    },
    "config": {
      "type": "object",
      "description": "Rule-specific configuration from .texide.json"
    },
    "source": {
      "type": "string",
      "description": "Full source text of the file"
    },
    "file_path": {
      "type": ["string", "null"],
      "description": "File path (if available)"
    }
  }
}
```

### LintResponse

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["diagnostics"],
  "properties": {
    "diagnostics": {
      "type": "array",
      "items": { "$ref": "#/$defs/Diagnostic" }
    }
  },
  "$defs": {
    "Diagnostic": {
      "type": "object",
      "required": ["rule_id", "message", "span"],
      "properties": {
        "rule_id": {
          "type": "string",
          "description": "Rule that generated this diagnostic"
        },
        "message": {
          "type": "string",
          "description": "Human-readable message"
        },
        "span": { "$ref": "#/$defs/Span" },
        "severity": {
          "type": "string",
          "enum": ["error", "warning", "info"],
          "default": "error"
        },
        "fix": { "$ref": "#/$defs/Fix" }
      }
    },
    "Span": {
      "type": "object",
      "required": ["start", "end"],
      "properties": {
        "start": { "type": "integer", "minimum": 0 },
        "end": { "type": "integer", "minimum": 0 }
      },
      "description": "Byte range [start, end)"
    },
    "Fix": {
      "type": "object",
      "required": ["span", "text"],
      "properties": {
        "span": { "$ref": "#/$defs/Span" },
        "text": {
          "type": "string",
          "description": "Replacement text (empty for deletion)"
        }
      }
    }
  }
}
```

## AST Node Types

Rules receive individual AST nodes based on their `node_types` manifest field.

### Block Elements

| Type | Description | Has Children |
|------|-------------|--------------|
| `Document` | Root node | Yes |
| `Paragraph` | Text paragraph | Yes |
| `Header` | Heading (h1-h6) | Yes |
| `BlockQuote` | Quote block | Yes |
| `List` | Ordered/unordered list | Yes |
| `ListItem` | List item | Yes |
| `CodeBlock` | Fenced code block | No |
| `HorizontalRule` | Thematic break | No |
| `Html` | Raw HTML block | No |
| `Table` | Table | Yes |
| `TableRow` | Table row | Yes |
| `TableCell` | Table cell | Yes |

### Inline Elements

| Type | Description | Has Children |
|------|-------------|--------------|
| `Str` | Plain text | No |
| `Break` | Line break | No |
| `Emphasis` | Italic text | Yes |
| `Strong` | Bold text | Yes |
| `Delete` | Strikethrough | Yes |
| `Code` | Inline code | No |
| `Link` | Hyperlink | Yes |
| `Image` | Image | No |
| `LinkReference` | Reference link | Yes |
| `ImageReference` | Reference image | No |
| `FootnoteReference` | Footnote ref | No |

## Example Implementations

### Rust (Extism PDK)

```rust
use extism_pdk::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct Manifest {
    name: &'static str,
    version: &'static str,
    description: &'static str,
    fixable: bool,
    node_types: Vec<&'static str>,
}

#[derive(Deserialize)]
struct LintRequest {
    node: serde_json::Value,
    config: serde_json::Value,
    source: String,
    file_path: Option<String>,
}

#[derive(Serialize)]
struct LintResponse {
    diagnostics: Vec<Diagnostic>,
}

#[derive(Serialize)]
struct Diagnostic {
    rule_id: String,
    message: String,
    span: Span,
    severity: String,
}

#[derive(Serialize)]
struct Span {
    start: u32,
    end: u32,
}

#[plugin_fn]
pub fn get_manifest() -> FnResult<String> {
    let manifest = Manifest {
        name: "my-rule",
        version: "1.0.0",
        description: "My custom rule",
        fixable: false,
        node_types: vec!["Str"],
    };
    Ok(serde_json::to_string(&manifest)?)
}

#[plugin_fn]
pub fn lint(input: String) -> FnResult<String> {
    let request: LintRequest = serde_json::from_str(&input)?;
    let diagnostics: Vec<Diagnostic> = vec![];

    // Your lint logic here

    Ok(serde_json::to_string(&LintResponse { diagnostics })?)
}
```

### AssemblyScript (Extism PDK)

```typescript
import { JSON } from "json-as";
import { Host, Output } from "@aspect/as-pdk";

@json
class Manifest {
  name: string = "my-rule";
  version: string = "1.0.0";
  description: string = "My custom rule";
  fixable: boolean = false;
  node_types: string[] = ["Str"];
}

@json
class Span {
  start: u32 = 0;
  end: u32 = 0;
}

@json
class Diagnostic {
  rule_id: string = "";
  message: string = "";
  span: Span = new Span();
  severity: string = "error";
}

@json
class LintResponse {
  diagnostics: Diagnostic[] = [];
}

export function get_manifest(): i32 {
  const manifest = new Manifest();
  Output.setString(JSON.stringify(manifest));
  return 0;
}

export function lint(): i32 {
  const input = Host.inputString();
  // Parse input and perform linting

  const response = new LintResponse();
  Output.setString(JSON.stringify(response));
  return 0;
}
```

## Security Considerations

- Rules run in a WASI sandbox with no filesystem or network access
- Rules cannot access host memory outside allocated regions
- Rules have execution time limits (configurable)
- All communication uses JSON serialization (no shared memory)

## Versioning

This specification follows semantic versioning. Breaking changes increment the major version.

| Spec Version | Changes |
|--------------|---------|
| 1.0.0 | Initial specification |
