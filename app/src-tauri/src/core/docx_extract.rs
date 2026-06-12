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

    let paragraphs = parse_paragraphs(&document_xml)?;
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
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                if e.name().as_ref() == b"w:p" {
                    in_paragraph = true;
                    current_para.clear();
                }
                if e.name().as_ref() == b"w:br" && in_paragraph {
                    current_para.push('\n');
                }
                if e.name().as_ref() == b"w:tab" && in_paragraph {
                    current_para.push('\t');
                }
            }
            Ok(Event::Text(ref e)) => {
                if in_paragraph {
                    let text = e.unescape().map_err(|e| format!("XML unescape error: {}", e))?;
                    current_para.push_str(&text);
                }
            }
            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == b"w:p" {
                    in_paragraph = false;
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
        if lower == *keyword || lower.trim_end_matches(':') == *keyword || lower.trim_end_matches('s') == *keyword {
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
