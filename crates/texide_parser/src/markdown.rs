//! Markdown parser using markdown-rs (wooorm/markdown-rs).
//!
//! This parser converts Markdown to TxtAST using the `markdown` crate,
//! which provides mdast-compatible AST output.

use markdown::{ParseOptions, to_mdast};
use texide_ast::{AstArena, NodeData, NodeType, Span, TxtNode};

use crate::{ParseError, Parser};

/// Markdown parser implementation.
///
/// Uses `markdown-rs` for parsing, which supports:
/// - CommonMark
/// - GFM (GitHub Flavored Markdown)
/// - MDX (optional)
/// - Math (optional)
/// - Frontmatter (optional)
pub struct MarkdownParser;

impl MarkdownParser {
    /// Creates a new Markdown parser with default options.
    pub fn new() -> Self {
        Self
    }

    /// Gets default parse options (GFM).
    fn default_options() -> ParseOptions {
        ParseOptions::gfm()
    }

    /// Converts an mdast node to TxtNode.
    fn convert_node<'a>(
        &self,
        arena: &'a AstArena,
        node: &markdown::mdast::Node,
        source: &str,
    ) -> TxtNode<'a> {
        use markdown::mdast::Node;

        match node {
            Node::Root(root) => {
                let children = self.convert_children(arena, &root.children, source);
                let span = self.node_span(node, source);
                TxtNode::new_parent(NodeType::Document, span, children)
            }

            Node::Paragraph(para) => {
                let children = self.convert_children(arena, &para.children, source);
                let span = self.node_span(node, source);
                TxtNode::new_parent(NodeType::Paragraph, span, children)
            }

            Node::Heading(heading) => {
                let children = self.convert_children(arena, &heading.children, source);
                let span = self.node_span(node, source);
                let mut node = TxtNode::new_parent(NodeType::Header, span, children);
                node.data = NodeData::header(heading.depth);
                node
            }

            Node::Text(text) => {
                let span = self.node_span(node, source);
                let value = arena.alloc_str(&text.value);
                TxtNode::new_text(NodeType::Str, span, value)
            }

            Node::Emphasis(em) => {
                let children = self.convert_children(arena, &em.children, source);
                let span = self.node_span(node, source);
                TxtNode::new_parent(NodeType::Emphasis, span, children)
            }

            Node::Strong(strong) => {
                let children = self.convert_children(arena, &strong.children, source);
                let span = self.node_span(node, source);
                TxtNode::new_parent(NodeType::Strong, span, children)
            }

            Node::InlineCode(code) => {
                let span = self.node_span(node, source);
                let value = arena.alloc_str(&code.value);
                TxtNode::new_text(NodeType::Code, span, value)
            }

            Node::Code(code) => {
                let span = self.node_span(node, source);
                let value = arena.alloc_str(&code.value);
                let mut node = TxtNode::new_text(NodeType::CodeBlock, span, value);
                if let Some(lang) = &code.lang {
                    node.data = NodeData::code_block(Some(arena.alloc_str(lang)));
                }
                node
            }

            Node::Link(link) => {
                let children = self.convert_children(arena, &link.children, source);
                let span = self.node_span(node, source);
                let mut node = TxtNode::new_parent(NodeType::Link, span, children);
                let url = arena.alloc_str(&link.url);
                let title = link.title.as_ref().map(|t| arena.alloc_str(t));
                node.data = NodeData::link(url, title);
                node
            }

            Node::Image(image) => {
                let span = self.node_span(node, source);
                let mut node = TxtNode::new_leaf(NodeType::Image, span);
                let url = arena.alloc_str(&image.url);
                let title = image.title.as_ref().map(|t| arena.alloc_str(t));
                node.data = NodeData::link(url, title);
                node
            }

            Node::List(list) => {
                let children = self.convert_children(arena, &list.children, source);
                let span = self.node_span(node, source);
                let mut node = TxtNode::new_parent(NodeType::List, span, children);
                node.data = NodeData::list(list.ordered);
                node
            }

            Node::ListItem(item) => {
                let children = self.convert_children(arena, &item.children, source);
                let span = self.node_span(node, source);
                TxtNode::new_parent(NodeType::ListItem, span, children)
            }

            Node::Blockquote(quote) => {
                let children = self.convert_children(arena, &quote.children, source);
                let span = self.node_span(node, source);
                TxtNode::new_parent(NodeType::BlockQuote, span, children)
            }

            Node::ThematicBreak(_) => {
                let span = self.node_span(node, source);
                TxtNode::new_leaf(NodeType::HorizontalRule, span)
            }

            Node::Break(_) => {
                let span = self.node_span(node, source);
                TxtNode::new_leaf(NodeType::Break, span)
            }

            Node::Html(html) => {
                let span = self.node_span(node, source);
                let value = arena.alloc_str(&html.value);
                TxtNode::new_text(NodeType::Html, span, value)
            }

            Node::Delete(del) => {
                let children = self.convert_children(arena, &del.children, source);
                let span = self.node_span(node, source);
                TxtNode::new_parent(NodeType::Delete, span, children)
            }

            // Table support (GFM)
            Node::Table(table) => {
                let children = self.convert_children(arena, &table.children, source);
                let span = self.node_span(node, source);
                TxtNode::new_parent(NodeType::Table, span, children)
            }

            Node::TableRow(row) => {
                let children = self.convert_children(arena, &row.children, source);
                let span = self.node_span(node, source);
                TxtNode::new_parent(NodeType::TableRow, span, children)
            }

            Node::TableCell(cell) => {
                let children = self.convert_children(arena, &cell.children, source);
                let span = self.node_span(node, source);
                TxtNode::new_parent(NodeType::TableCell, span, children)
            }

            // Footnotes (GFM)
            Node::FootnoteDefinition(def) => {
                let children = self.convert_children(arena, &def.children, source);
                let span = self.node_span(node, source);
                let mut node = TxtNode::new_parent(NodeType::FootnoteDefinition, span, children);
                node.data.identifier = Some(arena.alloc_str(&def.identifier));
                if let Some(label) = &def.label {
                    node.data.label = Some(arena.alloc_str(label));
                }
                node
            }

            Node::FootnoteReference(ref_node) => {
                let span = self.node_span(node, source);
                let mut node = TxtNode::new_leaf(NodeType::FootnoteReference, span);
                node.data.identifier = Some(arena.alloc_str(&ref_node.identifier));
                if let Some(label) = &ref_node.label {
                    node.data.label = Some(arena.alloc_str(label));
                }
                node
            }

            // Reference nodes
            Node::LinkReference(ref_node) => {
                let children = self.convert_children(arena, &ref_node.children, source);
                let span = self.node_span(node, source);
                let mut node = TxtNode::new_parent(NodeType::LinkReference, span, children);
                node.data.identifier = Some(arena.alloc_str(&ref_node.identifier));
                if let Some(label) = &ref_node.label {
                    node.data.label = Some(arena.alloc_str(label));
                }
                node
            }

            Node::ImageReference(ref_node) => {
                let span = self.node_span(node, source);
                let mut node = TxtNode::new_leaf(NodeType::ImageReference, span);
                node.data.identifier = Some(arena.alloc_str(&ref_node.identifier));
                if let Some(label) = &ref_node.label {
                    node.data.label = Some(arena.alloc_str(label));
                }
                node
            }

            Node::Definition(def) => {
                let span = self.node_span(node, source);
                let mut node = TxtNode::new_leaf(NodeType::Definition, span);
                node.data.identifier = Some(arena.alloc_str(&def.identifier));
                node.data.url = Some(arena.alloc_str(&def.url));
                if let Some(title) = &def.title {
                    node.data.title = Some(arena.alloc_str(title));
                }
                if let Some(label) = &def.label {
                    node.data.label = Some(arena.alloc_str(label));
                }
                node
            }

            // Fallback for unsupported nodes
            _ => {
                let span = self.node_span(node, source);
                TxtNode::new_leaf(NodeType::Html, span)
            }
        }
    }

    /// Converts a list of mdast children to TxtNode slice.
    fn convert_children<'a>(
        &self,
        arena: &'a AstArena,
        children: &[markdown::mdast::Node],
        source: &str,
    ) -> &'a [TxtNode<'a>] {
        let nodes: Vec<TxtNode<'a>> = children
            .iter()
            .map(|child| self.convert_node(arena, child, source))
            .collect();

        arena.alloc_slice_clone(&nodes)
    }

    /// Gets the span for an mdast node.
    fn node_span(&self, node: &markdown::mdast::Node, _source: &str) -> Span {
        if let Some(pos) = node.position() {
            Span::new(pos.start.offset as u32, pos.end.offset as u32)
        } else {
            Span::new(0, 0)
        }
    }
}

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for MarkdownParser {
    fn name(&self) -> &str {
        "markdown"
    }

    fn extensions(&self) -> &[&str] {
        &["md", "markdown", "mdown", "mkdn", "mkd"]
    }

    fn parse<'a>(&self, arena: &'a AstArena, source: &str) -> Result<TxtNode<'a>, ParseError> {
        let options = Self::default_options();
        let mdast =
            to_mdast(source, &options).map_err(|e| ParseError::invalid_source(e.to_string()))?;

        Ok(self.convert_node(arena, &mdast, source))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_markdown() {
        let arena = AstArena::new();
        let parser = MarkdownParser::new();
        let source = "# Hello\n\nThis is a paragraph.";

        let ast = parser.parse(&arena, source).unwrap();

        assert_eq!(ast.node_type, NodeType::Document);
        assert!(ast.has_children());
    }

    #[test]
    fn test_parse_heading() {
        let arena = AstArena::new();
        let parser = MarkdownParser::new();
        let source = "# Level 1\n\n## Level 2";

        let ast = parser.parse(&arena, source).unwrap();

        assert_eq!(ast.children.len(), 2);
        assert_eq!(ast.children[0].node_type, NodeType::Header);
        assert_eq!(ast.children[0].data.depth, Some(1));
        assert_eq!(ast.children[1].node_type, NodeType::Header);
        assert_eq!(ast.children[1].data.depth, Some(2));
    }

    #[test]
    fn test_parse_link() {
        let arena = AstArena::new();
        let parser = MarkdownParser::new();
        let source = "[Example](https://example.com)";

        let ast = parser.parse(&arena, source).unwrap();

        // Document > Paragraph > Link
        let paragraph = &ast.children[0];
        let link = &paragraph.children[0];

        assert_eq!(link.node_type, NodeType::Link);
        assert_eq!(link.data.url, Some("https://example.com"));
    }

    #[test]
    fn test_extensions() {
        let parser = MarkdownParser::new();

        assert!(parser.can_parse("md"));
        assert!(parser.can_parse("markdown"));
        assert!(parser.can_parse("MD"));
        assert!(!parser.can_parse("txt"));
    }
}
