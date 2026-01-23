# TDD Guide for Texide

## Red-Green-Refactor Cycle

### 1. Red: Write a Failing Test First

Write a test that describes the **behavior** you want, not the implementation.

```rust
#[test]
fn parses_heading_from_markdown() {
    // Arrange
    let source = "# Hello World";
    let arena = Arena::new();

    // Act
    let doc = parse_markdown(source, &arena);

    // Assert: Verify the behavior from the user's perspective
    let heading = doc.children().next().unwrap();
    assert_eq!(heading.node_type(), NodeType::Heading);
    assert_eq!(heading.raw_text(), "Hello World");
}
```

### 2. Green: Make the Test Pass

Write the **minimum code** necessary to make the test pass.

### 3. Refactor: Improve the Code

Clean up the implementation while keeping tests green.

## Behavior-Driven Testing Principles

### Test Behavior, Not Implementation

```rust
// Bad: Tests implementation details
#[test]
fn internal_buffer_has_correct_size() {
    let parser = Parser::new();
    assert_eq!(parser.buffer.len(), 1024);  // Implementation detail
}

// Good: Tests observable behavior
#[test]
fn parses_large_documents_without_error() {
    let large_content = "x".repeat(100_000);
    let result = Parser::parse(&large_content);
    assert!(result.is_ok());
}
```

### Use Descriptive Test Names

Test names should describe the behavior being tested:

```rust
// Pattern: <action>_<condition>_<expected_result>
fn lint_file_with_errors_returns_diagnostics()
fn parse_empty_string_returns_empty_document()
fn cache_expired_entry_triggers_reparse()
```

### Organize Tests by Behavior

```rust
mod parsing_headings {
    #[test]
    fn extracts_heading_text() { ... }

    #[test]
    fn preserves_heading_level() { ... }

    #[test]
    fn handles_empty_heading() { ... }
}

mod parsing_paragraphs {
    #[test]
    fn joins_multiple_lines() { ... }
}
```

## Test Libraries

### pretty_assertions

Better diff output for failed assertions:

```rust
use pretty_assertions::assert_eq;

#[test]
fn complex_struct_equality() {
    let expected = Document { /* ... */ };
    let actual = parse(input);
    assert_eq!(expected, actual);  // Shows colored diff on failure
}
```

### rstest

Parameterized tests and fixtures:

```rust
use rstest::rstest;

#[rstest]
#[case("# H1", 1)]
#[case("## H2", 2)]
#[case("### H3", 3)]
fn heading_level_is_correct(#[case] input: &str, #[case] expected_level: u8) {
    let doc = parse(input);
    assert_eq!(doc.heading_level(), expected_level);
}
```

### insta

Snapshot testing for complex output:

```rust
use insta::assert_json_snapshot;

#[test]
fn ast_structure_matches_snapshot() {
    let ast = parse("# Hello\n\nWorld");
    assert_json_snapshot!(ast);
}
```

### assert_cmd (CLI)

Behavior testing for CLI:

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn lint_shows_errors_in_output() {
    Command::cargo_bin("texide")
        .arg("lint")
        .arg("bad_file.md")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}
```

## Running Tests

```bash
# All tests
cargo test --workspace

# Specific crate
cargo test -p texide_ast

# Specific test
cargo test test_span

# With output
cargo test -- --nocapture

# Update snapshots (insta)
cargo insta test --review
```
