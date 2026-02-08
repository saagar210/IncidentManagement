//! Markdown → DOCX converter using pulldown-cmark.
//!
//! Converts a markdown string into a Vec<Paragraph> that can be appended to a Docx document.
//! Supports: bold, italic, code spans, headings, bullet lists, numbered lists, code blocks.

use docx_rs::*;
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

const BODY_SIZE: usize = 11 * 2; // 11pt in half-points
const CODE_SIZE: usize = 10 * 2;
const H3_SIZE: usize = 14 * 2;
const H4_SIZE: usize = 12 * 2;

/// Convert markdown text into DOCX paragraphs.
/// Falls back to plain text if markdown is trivial (no special syntax).
pub fn markdown_to_paragraphs(md: &str) -> Vec<Paragraph> {
    let trimmed = md.trim();
    if trimmed.is_empty() {
        return vec![];
    }

    let options = Options::empty();
    let parser = Parser::new_ext(trimmed, options);
    let events: Vec<Event> = parser.collect();

    let mut paragraphs: Vec<Paragraph> = Vec::new();
    let mut current_runs: Vec<Run> = Vec::new();
    let mut bold = false;
    let mut italic = false;
    let mut in_list = false;
    let mut ordered_list = false;
    let mut list_index: u64 = 0;
    let mut in_code_block = false;
    let mut code_block_text = String::new();
    let mut in_heading = false;
    let mut heading_level: u8 = 0;

    for event in events {
        match event {
            Event::Start(tag) => {
                match tag {
                    Tag::Heading { level, .. } => {
                        flush_paragraph(&mut paragraphs, &mut current_runs, false, false, 0);
                        in_heading = true;
                        heading_level = level as u8;
                    }
                    Tag::Paragraph => {
                        // Start of a new paragraph
                    }
                    Tag::Strong => bold = true,
                    Tag::Emphasis => italic = true,
                    Tag::List(start) => {
                        flush_paragraph(&mut paragraphs, &mut current_runs, false, false, 0);
                        in_list = true;
                        if let Some(s) = start {
                            ordered_list = true;
                            list_index = s;
                        } else {
                            ordered_list = false;
                            list_index = 0;
                        }
                    }
                    Tag::Item => {
                        flush_paragraph(&mut paragraphs, &mut current_runs, false, false, 0);
                    }
                    Tag::CodeBlock(_kind) => {
                        flush_paragraph(&mut paragraphs, &mut current_runs, false, false, 0);
                        in_code_block = true;
                        code_block_text.clear();
                    }
                    _ => {}
                }
            }
            Event::End(tag_end) => {
                match tag_end {
                    TagEnd::Heading(_level) => {
                        // Build heading paragraph
                        let size = match heading_level {
                            1 | 2 => H3_SIZE, // In sub-context, map h1/h2 down
                            3 => H3_SIZE,
                            _ => H4_SIZE,
                        };
                        let mut para = Paragraph::new();
                        for run in current_runs.drain(..) {
                            para = para.add_run(run.bold().size(size));
                        }
                        paragraphs.push(para);
                        in_heading = false;
                        heading_level = 0;
                    }
                    TagEnd::Paragraph => {
                        if in_heading {
                            continue;
                        }
                        flush_paragraph(&mut paragraphs, &mut current_runs, false, false, 0);
                    }
                    TagEnd::Strong => bold = false,
                    TagEnd::Emphasis => italic = false,
                    TagEnd::List(_) => {
                        in_list = false;
                        ordered_list = false;
                        list_index = 0;
                    }
                    TagEnd::Item => {
                        flush_paragraph(
                            &mut paragraphs,
                            &mut current_runs,
                            in_list,
                            ordered_list,
                            list_index,
                        );
                        if ordered_list {
                            list_index += 1;
                        }
                    }
                    TagEnd::CodeBlock => {
                        in_code_block = false;
                        // Render code block as a shaded paragraph with monospace-style text
                        for line in code_block_text.lines() {
                            let run = Run::new()
                                .add_text(line)
                                .size(CODE_SIZE)
                                .fonts(RunFonts::new().ascii("Courier New"));
                            paragraphs.push(
                                Paragraph::new().add_run(run)
                            );
                        }
                        code_block_text.clear();
                    }
                    _ => {}
                }
            }
            Event::Text(text) => {
                if in_code_block {
                    code_block_text.push_str(&text);
                } else {
                    let mut run = Run::new().add_text(text.as_ref()).size(BODY_SIZE);
                    if bold {
                        run = run.bold();
                    }
                    if italic {
                        run = run.italic();
                    }
                    current_runs.push(run);
                }
            }
            Event::Code(code) => {
                // Inline code
                let run = Run::new()
                    .add_text(code.as_ref())
                    .size(CODE_SIZE)
                    .fonts(RunFonts::new().ascii("Courier New"));
                current_runs.push(run);
            }
            Event::SoftBreak | Event::HardBreak => {
                // Treat as paragraph break
                flush_paragraph(&mut paragraphs, &mut current_runs, in_list, ordered_list, list_index);
            }
            _ => {}
        }
    }

    // Flush remaining
    flush_paragraph(&mut paragraphs, &mut current_runs, false, false, 0);

    paragraphs
}

fn flush_paragraph(
    paragraphs: &mut Vec<Paragraph>,
    runs: &mut Vec<Run>,
    is_list_item: bool,
    is_ordered: bool,
    list_index: u64,
) {
    if runs.is_empty() {
        return;
    }

    let mut para = Paragraph::new();

    if is_list_item {
        let prefix = if is_ordered {
            format!("{}. ", list_index)
        } else {
            "\u{2022}  ".to_string()
        };
        para = para.add_run(Run::new().add_text(prefix).size(BODY_SIZE));
    }

    for run in runs.drain(..) {
        para = para.add_run(run);
    }

    paragraphs.push(para);
}

/// Convert markdown to DOCX paragraphs and append to a Docx.
/// If the input is empty, returns the docx unchanged.
pub fn append_markdown(mut docx: Docx, md: &str) -> Docx {
    for para in markdown_to_paragraphs(md) {
        docx = docx.add_paragraph(para);
    }
    docx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_returns_empty() {
        let result = markdown_to_paragraphs("");
        assert!(result.is_empty());
    }

    #[test]
    fn plain_text_produces_one_paragraph() {
        let result = markdown_to_paragraphs("Hello world");
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn bullet_list_produces_multiple_paragraphs() {
        let md = "- Item one\n- Item two\n- Item three";
        let result = markdown_to_paragraphs(md);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn code_block_renders() {
        let md = "```\nlet x = 1;\nlet y = 2;\n```";
        let result = markdown_to_paragraphs(md);
        assert!(result.len() >= 2); // At least 2 lines
    }
}
