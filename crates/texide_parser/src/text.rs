//! Plain text parser.
//!
//! This parser treats plain text as a simple document with paragraphs.

use texide_ast::{AstArena, NodeType, Span, TxtNode};

use crate::{ParseError, Parser};

/// Plain text parser implementation.
///
/// Parses plain text files into TxtAST. The text is split into paragraphs
/// by blank lines, and each paragraph contains text nodes.
pub struct PlainTextParser;

impl PlainTextParser {
    /// Creates a new plain text parser.
    pub fn new() -> Self {
        Self
    }
}

impl Default for PlainTextParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for PlainTextParser {
    fn name(&self) -> &str {
        "text"
    }

    fn extensions(&self) -> &[&str] {
        &["txt", "text"]
    }

    fn parse<'a>(&self, arena: &'a AstArena, source: &str) -> Result<TxtNode<'a>, ParseError> {
        let mut paragraphs: Vec<TxtNode<'a>> = Vec::new();
        let mut current_start: Option<usize> = None;
        let mut current_end: usize = 0;

        for (idx, line) in source.lines().enumerate() {
            let line_start = if idx == 0 {
                0
            } else {
                // Find the actual byte offset of this line
                source[..current_end]
                    .rfind('\n')
                    .map(|p| p + 1)
                    .unwrap_or(current_end)
            };

            // Update current_end to include this line
            current_end = line_start + line.len();
            if current_end < source.len() && source.as_bytes().get(current_end) == Some(&b'\n') {
                current_end += 1;
            }

            if line.trim().is_empty() {
                // End of paragraph
                if let Some(start) = current_start {
                    let para_text = &source[start..line_start.saturating_sub(1).max(start)];
                    if !para_text.trim().is_empty() {
                        let text_node = arena.alloc(TxtNode::new_text(
                            NodeType::Str,
                            Span::new(start as u32, (start + para_text.len()) as u32),
                            arena.alloc_str(para_text),
                        ));
                        let children = arena.alloc_slice_copy(&[*text_node]);
                        paragraphs.push(TxtNode::new_parent(
                            NodeType::Paragraph,
                            Span::new(start as u32, (start + para_text.len()) as u32),
                            children,
                        ));
                    }
                    current_start = None;
                }
            } else if current_start.is_none() {
                // Start of new paragraph
                current_start = Some(line_start);
            }
        }

        // Handle final paragraph
        if let Some(start) = current_start {
            let para_text = source[start..].trim_end();
            if !para_text.is_empty() {
                let text_node = arena.alloc(TxtNode::new_text(
                    NodeType::Str,
                    Span::new(start as u32, (start + para_text.len()) as u32),
                    arena.alloc_str(para_text),
                ));
                let children = arena.alloc_slice_copy(&[*text_node]);
                paragraphs.push(TxtNode::new_parent(
                    NodeType::Paragraph,
                    Span::new(start as u32, (start + para_text.len()) as u32),
                    children,
                ));
            }
        }

        let children = arena.alloc_slice_clone(&paragraphs);
        Ok(TxtNode::new_parent(
            NodeType::Document,
            Span::new(0, source.len() as u32),
            children,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_text() {
        let arena = AstArena::new();
        let parser = PlainTextParser::new();
        let source = "Hello, world!";

        let ast = parser.parse(&arena, source).unwrap();

        assert_eq!(ast.node_type, NodeType::Document);
        assert_eq!(ast.children.len(), 1);
        assert_eq!(ast.children[0].node_type, NodeType::Paragraph);
    }

    #[test]
    fn test_parse_multiple_paragraphs() {
        let arena = AstArena::new();
        let parser = PlainTextParser::new();
        let source = "First paragraph.\n\nSecond paragraph.";

        let ast = parser.parse(&arena, source).unwrap();

        assert_eq!(ast.node_type, NodeType::Document);
        assert_eq!(ast.children.len(), 2);
    }

    #[test]
    fn test_parse_empty_text() {
        let arena = AstArena::new();
        let parser = PlainTextParser::new();
        let source = "";

        let ast = parser.parse(&arena, source).unwrap();

        assert_eq!(ast.node_type, NodeType::Document);
        assert!(ast.children.is_empty());
    }

    #[test]
    fn test_extensions() {
        let parser = PlainTextParser::new();

        assert!(parser.can_parse("txt"));
        assert!(parser.can_parse("text"));
        assert!(parser.can_parse("TXT"));
        assert!(!parser.can_parse("md"));
    }
}
