// SPDX-License-Identifier: PMPL-1.0-or-later
//! A2ML parser implementation using nom
//!
//! Parses A2ML Module 0 surface syntax:
//! - Directives: @abstract, @refs, @requires, @end
//! - Headings: # Level 1, ## Level 2, etc.
//! - Paragraphs, lists, tables
//! - Inline formatting: *emphasis*, **strong**, [links](url)
//! - References: [1], [2]

use crate::error::{PolicyError, Result};
use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, take_until, take_while, take_while1, is_not},
    character::complete::{char, line_ending, multispace0, multispace1, not_line_ending, space0, space1},
    combinator::{map, opt, recognize, value},
    multi::{many0, many1, separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, terminated, tuple},
};
use std::path::Path;
use std::fs;

/// Represents a parsed A2ML document
#[derive(Debug, Clone, serde::Serialize)]
pub struct A2mlDocument {
    /// Document abstract
    pub abstract_text: Option<String>,

    /// Document sections
    pub sections: Vec<Section>,

    /// References
    pub references: Vec<Reference>,

    /// Requirements (external dependencies)
    pub requirements: Vec<String>,

    /// Raw source text (for preservation)
    pub raw: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Section {
    /// Section heading text
    pub heading: String,

    /// Section level (1-6, like Markdown)
    pub level: u8,

    /// Section content (paragraphs, lists, etc.)
    pub content: Vec<ContentBlock>,

    /// Attestations in this section
    pub attestations: Vec<Attestation>,

    /// Line number where section starts
    pub line_number: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum ContentBlock {
    Paragraph(String),
    BulletList(Vec<String>),
    Table { headers: Vec<String>, rows: Vec<Vec<String>> },
    CodeBlock { language: Option<String>, code: String },
    HorizontalRule,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Attestation {
    /// Claim being attested
    pub claim: String,

    /// What must be verified
    pub requirement: String,

    /// External reference (e.g., "NUJ Code §1")
    pub reference: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Reference {
    /// Reference ID (e.g., "1")
    pub id: String,

    /// Reference text
    pub text: String,

    /// URL (if applicable)
    pub url: Option<String>,
}

/// Parse an A2ML file
pub fn parse_a2ml_file(path: &Path) -> Result<A2mlDocument> {
    log::debug!("Parsing A2ML file: {:?}", path);

    if !path.exists() {
        return Err(PolicyError::FileNotFound(path.to_path_buf()));
    }

    let content = fs::read_to_string(path)?;
    parse_a2ml_string(&content)
}

/// Parse A2ML from a string
pub fn parse_a2ml_string(content: &str) -> Result<A2mlDocument> {
    log::debug!("Parsing A2ML from string ({} bytes)", content.len());

    match document(content) {
        Ok((_, doc)) => Ok(doc),
        Err(e) => {
            let error_msg = match e {
                nom::Err::Error(e) | nom::Err::Failure(e) => {
                    format!("Parse error at: {}", e.input.chars().take(50).collect::<String>())
                }
                nom::Err::Incomplete(_) => "Incomplete input".to_string(),
            };
            Err(PolicyError::ParseError(error_msg))
        }
    }
}

// ============================================================================
// Parser Combinators
// ============================================================================

/// Parse a complete A2ML document
fn document(input: &str) -> IResult<&str, A2mlDocument> {
    let (input, _) = multispace0(input)?;

    // Parse abstract (optional)
    let (input, abstract_text) = opt(abstract_directive)(input)?;
    let (input, _) = multispace0(input)?;

    // Parse requires (optional)
    let (input, requirements) = opt(requires_directive)(input)?;
    let (input, _) = multispace0(input)?;

    // Parse sections
    let (input, sections) = many0(section)(input)?;
    let (input, _) = multispace0(input)?;

    // Parse references (optional)
    let (input, references) = opt(refs_directive)(input)?;
    let (input, _) = multispace0(input)?;

    Ok((input, A2mlDocument {
        abstract_text,
        sections,
        references: references.unwrap_or_default(),
        requirements: requirements.unwrap_or_default(),
        raw: input.to_string(),
    }))
}

/// Parse @abstract: ... @end
fn abstract_directive(input: &str) -> IResult<&str, String> {
    let (input, _) = tag("@abstract:")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, content) = take_until("@end")(input)?;
    let (input, _) = tag("@end")(input)?;
    let (input, _) = multispace0(input)?;

    Ok((input, content.trim().to_string()))
}

/// Parse @requires: ... @end
fn requires_directive(input: &str) -> IResult<&str, Vec<String>> {
    let (input, _) = tag("@requires:")(input)?;
    let (input, _) = multispace0(input)?;

    let (input, items) = many1(terminated(
        preceded(
            tuple((char('-'), space0)),
            map(not_line_ending, |s: &str| s.trim().to_string())
        ),
        line_ending
    ))(input)?;

    let (input, _) = tag("@end")(input)?;
    let (input, _) = multispace0(input)?;

    Ok((input, items))
}

/// Parse @refs: ... @end
fn refs_directive(input: &str) -> IResult<&str, Vec<Reference>> {
    let (input, _) = tag("@refs:")(input)?;
    let (input, _) = multispace0(input)?;

    let (input, refs) = many1(reference)(input)?;

    let (input, _) = tag("@end")(input)?;
    let (input, _) = multispace0(input)?;

    Ok((input, refs))
}

/// Parse a single reference: [1] Text here
fn reference(input: &str) -> IResult<&str, Reference> {
    let (input, _) = char('[')(input)?;
    let (input, id) = take_while1(|c: char| c.is_numeric())(input)?;
    let (input, _) = char(']')(input)?;
    let (input, _) = space0(input)?;
    let (input, text) = not_line_ending(input)?;
    let (input, _) = line_ending(input)?;

    // Check if URL in text (simple heuristic)
    let text_str = text.trim();
    let (text_final, url) = if text_str.contains("http://") || text_str.contains("https://") {
        // Extract URL (simplified - just find first http URL)
        if let Some(start) = text_str.find("http") {
            let url_part = &text_str[start..];
            let url_end = url_part.find(|c: char| c.is_whitespace() || c == ')').unwrap_or(url_part.len());
            let url = url_part[..url_end].to_string();
            (text_str[..start].trim().to_string(), Some(url))
        } else {
            (text_str.to_string(), None)
        }
    } else {
        (text_str.to_string(), None)
    };

    Ok((input, Reference {
        id: id.to_string(),
        text: text_final,
        url,
    }))
}

/// Parse a section (heading + content)
fn section(input: &str) -> IResult<&str, Section> {
    let (input, (level, heading)) = heading(input)?;
    let (input, _) = multispace0(input)?;

    // Parse content blocks until next heading or end
    let (input, blocks) = many0(terminated(content_block, multispace0))(input)?;

    // Extract attestations from content
    let attestations = extract_attestations(&blocks);

    Ok((input, Section {
        heading,
        level,
        content: blocks,
        attestations,
        line_number: 0,  // TODO: Track line numbers
    }))
}

/// Parse a heading: # Level 1, ## Level 2, etc.
fn heading(input: &str) -> IResult<&str, (u8, String)> {
    let (input, hashes) = take_while1(|c| c == '#')(input)?;
    let (input, _) = space1(input)?;
    let (input, text) = not_line_ending(input)?;
    let (input, _) = line_ending(input)?;

    let level = hashes.len().min(6) as u8;

    Ok((input, (level, text.trim().to_string())))
}

/// Parse a content block (paragraph, list, table, etc.)
fn content_block(input: &str) -> IResult<&str, ContentBlock> {
    alt((
        horizontal_rule,
        bullet_list,
        code_block,
        paragraph,
    ))(input)
}

/// Check if a line is a paragraph line (not a heading, list, or other structure)
fn is_paragraph_line(input: &str) -> bool {
    if input.is_empty() {
        return false;
    }

    let first_char = input.chars().next().unwrap();

    // Not a heading
    if first_char == '#' {
        return false;
    }

    // Not a horizontal rule
    if input.starts_with("---") {
        return false;
    }

    // Not a code block
    if input.starts_with("```") {
        return false;
    }

    // Not a directive
    if input.starts_with("@") {
        return false;
    }

    // Not a list item (- followed by space)
    if input.starts_with("- ") {
        return false;
    }

    true
}

/// Parse a paragraph line (not a heading or structural element)
fn paragraph_line(input: &str) -> IResult<&str, &str> {
    // Check if this looks like a paragraph line
    if !is_paragraph_line(input) {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Verify)));
    }

    not_line_ending(input)
}

/// Parse a paragraph
fn paragraph(input: &str) -> IResult<&str, ContentBlock> {
    let (input, lines) = many1(terminated(paragraph_line, line_ending))(input)?;

    // Join lines and trim
    let text = lines.join("\n").trim().to_string();

    // Skip empty paragraphs
    if text.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Verify)));
    }

    Ok((input, ContentBlock::Paragraph(text)))
}

/// Parse a bullet list
fn bullet_list(input: &str) -> IResult<&str, ContentBlock> {
    let (input, items) = many1(list_item)(input)?;
    Ok((input, ContentBlock::BulletList(items)))
}

/// Parse a single list item: - Item text
fn list_item(input: &str) -> IResult<&str, String> {
    let (input, _) = char('-')(input)?;
    let (input, _) = space1(input)?;
    let (input, text) = not_line_ending(input)?;
    let (input, _) = line_ending(input)?;

    Ok((input, text.trim().to_string()))
}

/// Parse a horizontal rule: ---
fn horizontal_rule(input: &str) -> IResult<&str, ContentBlock> {
    let (input, _) = tag("---")(input)?;
    let (input, _) = line_ending(input)?;
    Ok((input, ContentBlock::HorizontalRule))
}

/// Parse a code block: ```language ... ```
fn code_block(input: &str) -> IResult<&str, ContentBlock> {
    let (input, _) = tag("```")(input)?;
    let (input, language) = opt(map(not_line_ending, |s: &str| s.trim().to_string()))(input)?;
    let (input, _) = line_ending(input)?;
    let (input, code) = take_until("```")(input)?;
    let (input, _) = tag("```")(input)?;
    let (input, _) = opt(line_ending)(input)?;

    Ok((input, ContentBlock::CodeBlock {
        language,
        code: code.to_string(),
    }))
}

/// Extract attestations from content blocks (simple keyword search)
fn extract_attestations(blocks: &[ContentBlock]) -> Vec<Attestation> {
    let mut attestations = Vec::new();

    for block in blocks {
        if let ContentBlock::Paragraph(text) = block {
            // Look for "Attestation:" keyword
            if text.contains("**Attestation:**") || text.contains("Attestation:") {
                // Extract attestation text (simplified)
                let parts: Vec<&str> = text.split("Attestation:").collect();
                if parts.len() > 1 {
                    let attestation_text = parts[1].trim();

                    // Parse out "Must/Should/Could"
                    let requirement = if attestation_text.starts_with("*Must*") {
                        "MUST"
                    } else if attestation_text.starts_with("*Should*") {
                        "SHOULD"
                    } else if attestation_text.starts_with("*Could*") {
                        "COULD"
                    } else {
                        "MUST"  // Default
                    };

                    attestations.push(Attestation {
                        claim: text.lines().next().unwrap_or("").to_string(),
                        requirement: requirement.to_string(),
                        reference: None,  // TODO: Extract references
                    });
                }
            }
        }
    }

    attestations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_abstract() {
        let input = r#"@abstract:
This is a test abstract.
It has multiple lines.
@end

"#;
        let result = abstract_directive(input);
        assert!(result.is_ok());
        let (_, abstract_text) = result.unwrap();
        assert!(abstract_text.contains("test abstract"));
    }

    #[test]
    fn test_parse_requires() {
        let input = r#"@requires:
- UK Employment Rights Act 1996
- GDPR (EU 2016/679)
@end

"#;
        let result = requires_directive(input);
        assert!(result.is_ok());
        let (_, requirements) = result.unwrap();
        assert_eq!(requirements.len(), 2);
        assert_eq!(requirements[0], "UK Employment Rights Act 1996");
    }

    #[test]
    fn test_parse_reference() {
        let input = "[1] UK Employment Rights Act 1996\n";
        let result = reference(input);
        assert!(result.is_ok());
        let (_, ref_) = result.unwrap();
        assert_eq!(ref_.id, "1");
        assert!(ref_.text.contains("Employment Rights Act"));
    }

    #[test]
    fn test_parse_heading() {
        let input = "## Section Title\n";
        let result = heading(input);
        assert!(result.is_ok());
        let (_, (level, text)) = result.unwrap();
        assert_eq!(level, 2);
        assert_eq!(text, "Section Title");
    }

    #[test]
    fn test_parse_paragraph() {
        let input = "This is a paragraph.\nIt has two lines.\n\n";
        let result = paragraph(input);
        assert!(result.is_ok());
        let (_, block) = result.unwrap();
        if let ContentBlock::Paragraph(text) = block {
            assert!(text.contains("paragraph"));
        } else {
            panic!("Expected paragraph");
        }
    }

    #[test]
    fn test_parse_bullet_list() {
        let input = "- Item 1\n- Item 2\n- Item 3\n\n";
        let result = bullet_list(input);
        assert!(result.is_ok());
        let (_, block) = result.unwrap();
        if let ContentBlock::BulletList(items) = block {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0], "Item 1");
        } else {
            panic!("Expected bullet list");
        }
    }

    #[test]
    fn test_parse_simple_document() {
        let a2ml = r#"
@abstract:
This is a test contract.
@end

@requires:
- UK Employment Rights Act 1996
@end

## Section 1

This is a paragraph.

- Item 1
- Item 2

**Attestation:** *Must* comply with UK law.

@refs:
[1] UK Employment Rights Act 1996
@end
        "#;

        let result = parse_a2ml_string(a2ml);
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert!(doc.abstract_text.is_some());
        assert_eq!(doc.requirements.len(), 1);
        assert_eq!(doc.references.len(), 1);
        assert!(doc.sections.len() > 0);
    }
}
