use std::fs::File;
use std::io::Read;

use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use zip::ZipArchive;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ManuscriptText {
    pub raw_text: String,
    pub sections: Vec<Section>,
    pub paragraphs: Vec<String>,
    pub paragraph_count: usize,
    pub char_count: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Section {
    pub heading: String,
    pub body: String,
}

const HEADING_KEYWORDS: &[&str] = &[
    "abstract",
    "introduction",
    "background",
    "aim",
    "aims",
    "objective",
    "objectives",
    "methods",
    "materials and methods",
    "participants and methods",
    "results",
    "discussion",
    "conclusion",
    "conclusions",
    "references",
    "acknowledgments",
    "acknowledgements",
];

// Tags whose text content should be excluded from extraction
const EXCLUDE_TAGS: &[&[u8]] = &[
    b"w:instrText",      // Field codes (HYPERLINK, REF, CITATION, etc.)
    b"w:delText",        // Deleted text (track changes)
    b"w:commentReference", // Comment references
    b"w:footnoteRef",    // Footnote references
    b"w:endnoteRef",     // Endnote references
    b"w:fldChar",        // Field character boundaries
    b"w:bookmarkStart",  // Bookmarks
    b"w:bookmarkEnd",
    b"w:smartTag",       // Smart tags
];

pub fn extract_docx(path: &str) -> Result<ManuscriptText, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
    let mut archive =
        ZipArchive::new(file).map_err(|e| format!("Failed to read as zip: {}", e))?;

    let mut document_xml = String::new();
    archive
        .by_name("word/document.xml")
        .map_err(|e| format!("word/document.xml not found: {}", e))?
        .read_to_string(&mut document_xml)
        .map_err(|e| format!("Failed to read document.xml: {}", e))?;

    let mut paragraphs = parse_paragraphs(&document_xml)?;
    paragraphs.retain(|p| !is_garbage(p));
    let sections = detect_sections(&paragraphs);
    let raw_text = paragraphs.join("\n\n");
    let char_count = raw_text.chars().count();
    let paragraph_count = paragraphs.len();

    Ok(ManuscriptText {
        raw_text,
        sections,
        paragraphs,
        paragraph_count,
        char_count,
    })
}

fn parse_paragraphs(xml: &str) -> Result<Vec<String>, String> {
    let mut reader = Reader::from_str(xml);
    let mut paragraphs: Vec<String> = Vec::new();
    let mut current_para = String::new();
    let mut in_paragraph = false;
    let mut skip_depth: u32 = 0; // >0 means we're inside an excluded element
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let qname = e.name();
                let name_bytes: &[u8] = qname.as_ref();
                let is_exclude = EXCLUDE_TAGS.contains(&name_bytes);

                if name_bytes == b"w:p" {
                    in_paragraph = true;
                    current_para.clear();
                    skip_depth = 0;
                }

                if in_paragraph && is_exclude {
                    skip_depth += 1;
                }

                if skip_depth == 0 && in_paragraph {
                    if name_bytes == b"w:br" {
                        current_para.push('\n');
                    } else if name_bytes == b"w:tab" {
                        current_para.push('\t');
                    }
                }
            }
            Ok(Event::Empty(ref e)) => {
                let qname = e.name();
                let name_bytes: &[u8] = qname.as_ref();
                if skip_depth == 0 && in_paragraph {
                    if name_bytes == b"w:br" {
                        current_para.push('\n');
                    } else if name_bytes == b"w:tab" {
                        current_para.push('\t');
                    }
                }
            }
            Ok(Event::Text(ref e)) => {
                if in_paragraph && skip_depth == 0 {
                    let text = e.unescape().map_err(|e| format!("XML unescape error: {}", e))?;
                    current_para.push_str(&text);
                }
            }
            Ok(Event::End(ref e)) => {
                let qname = e.name();
                let name_bytes: &[u8] = qname.as_ref();
                let is_exclude = EXCLUDE_TAGS.contains(&name_bytes);

                if is_exclude && skip_depth > 0 {
                    skip_depth -= 1;
                }

                if name_bytes == b"w:p" {
                    in_paragraph = false;
                    skip_depth = 0;
                    let trimmed = current_para.trim().to_string();
                    if !trimmed.is_empty() {
                        paragraphs.push(trimmed);
                    }
                    current_para.clear();
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("XML parse error: {}", e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(paragraphs)
}

/// Filter out paragraphs that are clearly not natural text.
fn is_garbage(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return true;
    }

    // Very long alphanumeric-only strings (Base64, embedded data)
    // Natural text has spaces, punctuation, mixed case patterns
    let alphanumeric_only: String = trimmed.chars().filter(|c| c.is_alphanumeric()).collect();
    if alphanumeric_only.len() > 200 {
        let space_count = trimmed.chars().filter(|c| *c == ' ').count();
        let ratio = space_count as f64 / trimmed.len() as f64;
        // Natural English text has ~15-20% spaces; Base64/garbage has near 0%
        if ratio < 0.02 {
            return true;
        }
    }

    // Single extremely long "word" (no spaces) > 100 chars
    if !trimmed.contains(' ') && trimmed.len() > 100 {
        return true;
    }

    // Looks like XML fragments
    if trimmed.starts_with("<?xml") || trimmed.starts_with("<w:") || trimmed.starts_with("</w:") {
        return true;
    }

    // Looks like field code remnants
    if trimmed.starts_with("REF ") || trimmed.starts_with("HYPERLINK") || trimmed.starts_with("CITATION") {
        return true;
    }

    // Mostly non-printable or special characters
    let normal_chars = trimmed.chars().filter(|c| c.is_ascii_alphanumeric() || c.is_whitespace() || ".,;:!?()-/\"'[]{}@#$%^&*+=_~<>".contains(*c)).count();
    if (normal_chars as f64 / trimmed.len() as f64) < 0.5 {
        return true;
    }

    false
}

fn detect_sections(paragraphs: &[String]) -> Vec<Section> {
    let mut sections: Vec<Section> = Vec::new();
    let mut current_heading = String::new();
    let mut current_body = String::new();

    for para in paragraphs {
        let lower = para.to_lowercase();
        let trimmed = para.trim();

        if is_heading_candidate(trimmed, &lower) {
            if !current_heading.is_empty() || !current_body.is_empty() {
                sections.push(Section {
                    heading: if current_heading.is_empty() {
                        "Body".to_string()
                    } else {
                        current_heading.clone()
                    },
                    body: current_body.trim().to_string(),
                });
            }
            current_heading = trimmed.to_string();
            current_body.clear();
        } else {
            if !current_body.is_empty() {
                current_body.push_str("\n\n");
            }
            current_body.push_str(para);
        }
    }

    if !current_heading.is_empty() || !current_body.is_empty() {
        sections.push(Section {
            heading: if current_heading.is_empty() {
                "Body".to_string()
            } else {
                current_heading
            },
            body: current_body.trim().to_string(),
        });
    }

    if sections.is_empty() && !paragraphs.is_empty() {
        sections.push(Section {
            heading: "Full Text".to_string(),
            body: paragraphs.join("\n\n"),
        });
    }

    sections
}

fn is_heading_candidate(trimmed: &str, lower: &str) -> bool {
    if trimmed.is_empty() || trimmed.len() > 120 {
        return false;
    }

    for keyword in HEADING_KEYWORDS {
        if lower == *keyword
            || lower.trim_end_matches(':') == *keyword
            || lower.trim_end_matches('s') == *keyword
        {
            return true;
        }
    }

    if trimmed.len() < 80
        && trimmed.ends_with(':')
        && !trimmed.contains(". ")
        && trimmed.chars().filter(|c| c.is_uppercase()).count() as f64 / trimmed.len() as f64 > 0.5
    {
        return true;
    }

    false
}
