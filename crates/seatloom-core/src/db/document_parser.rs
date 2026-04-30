/// Document parser for SeatLoom typed coordination documents.
///
/// Extracts universal header fields (DOCUMENT_TEMPLATES.md §2) from markdown
/// frontmatter tables and produces deterministic section/anchor projections.
///
/// This is a pure-Rust, sync, no-DB parser — it operates on file content strings
/// and produces structured data ready for PostgreSQL insertion.
use std::path::Path;

/// Parsed universal header from a DOCUMENT_TEMPLATES.md §2 header table.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedDocHeader {
    pub template: Option<String>,
    pub subtype: Option<String>,
    pub doc_id: Option<String>,
    pub title: Option<String>,
    pub status: Option<String>,
    pub author: Option<String>,
    pub date: Option<String>,
    pub version: Option<String>,
    pub depends_on: Vec<String>,
    pub supersedes: Option<String>,
    pub tags: Vec<String>,
}

/// A single heading/section extracted from a markdown document.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedSection {
    /// 1-based ordinal position in the document.
    pub ordinal: usize,
    pub heading_text: String,
    pub heading_level: usize,
    /// Stable anchor slug for in-document linking.
    pub anchor_slug: String,
    /// First 512 chars of section body (text after heading, before next same-or-higher heading).
    pub body_excerpt: Option<String>,
    /// Normalized searchable text (heading + excerpt).
    pub search_text: String,
}

// =============================================================================
// Header parsing
// =============================================================================

/// Parse the universal header from a SeatLoom coordination document body.
///
/// Looks for a YAML-compatible markdown table after the title H1, following
/// the DOCUMENT_TEMPLATES.md §2 schema. Always returns `Some`; fields are
/// `None` / empty when the header table is absent. The title is populated
/// from the first H1 heading regardless of whether a table exists.
pub fn parse_header(body: &str) -> Option<ParsedDocHeader> {
    let lines = non_fenced_lines(body);

    let mut header = ParsedDocHeader {
        template: None,
        subtype: None,
        doc_id: None,
        title: None,
        status: None,
        author: None,
        date: None,
        version: None,
        depends_on: vec![],
        supersedes: None,
        tags: vec![],
    };

    // Extract the H1 title (first # heading)
    if let Some(title_line) = lines.iter().find(|l| l.starts_with("# ")) {
        header.title = Some(title_line.trim_start_matches('#').trim().to_string());
    }

    // Find the header table — look for "| Field | Value |" row
    let table_start = lines.iter().position(|l| {
        let trimmed = l.trim();
        (trimmed.starts_with("| Field") || trimmed.starts_with("| field"))
            && trimmed.contains("Value")
    });

    // If no table found, return header with title only
    let table_start = match table_start {
        Some(idx) => idx,
        None => return Some(header),
    };

    // Parse table rows after the separator
    for line in lines.iter().skip(table_start + 2) {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') || trimmed == "|---|---|" || trimmed == "| --- | --- |" {
            if !trimmed.starts_with('|') {
                break;
            }
            continue;
        }
        let parts: Vec<&str> = trimmed
            .trim_matches('|')
            .split('|')
            .map(|s| s.trim())
            .collect();
        if parts.len() < 2 {
            continue;
        }
        let key = parts[0].to_lowercase();
        let value = parts[1].to_string();
        if value.is_empty() || value == "—" || value == "-" {
            continue;
        }
        match key.as_str() {
            "template" => {
                header.template = normalize_template(&value)
                    .map(str::to_string)
                    .or(Some(value));
            }
            "subtype" => header.subtype = Some(value),
            "id" => header.doc_id = Some(value),
            "title" if header.title.is_none() => {
                header.title = Some(value);
            }
            "title" => {}
            "status" => header.status = Some(value),
            "author" => header.author = Some(value),
            "date" => header.date = Some(value),
            "version" => header.version = Some(value),
            "depends_on" => {
                header.depends_on = parse_list_value(&value);
            }
            "supersedes" => header.supersedes = Some(value),
            "tags" => {
                header.tags = parse_list_value(&value);
            }
            _ => {}
        }
    }

    Some(header)
}

fn non_fenced_lines(body: &str) -> Vec<&str> {
    let mut lines = Vec::new();
    let mut in_fence = false;

    for line in body.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if !in_fence {
            lines.push(line);
        }
    }

    lines
}

/// Parse a backtick-quoted or comma-separated list value from a table cell.
fn parse_list_value(value: &str) -> Vec<String> {
    if value.trim().is_empty() {
        return vec![];
    }
    // Handle backtick-quoted comma-separated items like "`a`, `b`, `c`"
    let normalized = value.replace('`', "");
    normalized
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

// =============================================================================
// Section extraction
// =============================================================================

/// Extract all heading sections from a markdown document body.
/// Produces deterministic anchor slugs and body excerpts.
pub fn extract_sections(body: &str) -> Vec<ParsedSection> {
    let lines: Vec<&str> = body.lines().collect();
    let mut sections: Vec<(usize, usize, String)> = Vec::new(); // (line_idx, level, text)

    for (i, line) in lines.iter().enumerate() {
        if let Some((level, text)) = parse_heading(line) {
            sections.push((i, level, text));
        }
    }

    let mut result = Vec::new();
    for (ordinal, (line_idx, level, heading_text)) in sections.iter().enumerate() {
        let anchor = slugify(heading_text);

        // Extract body: lines between this heading and the next heading of same/higher level
        let next_heading_line = sections
            .iter()
            .skip(ordinal + 1)
            .find(|(_, next_level, _)| *next_level <= *level)
            .map(|(l, _, _)| *l)
            .unwrap_or(lines.len());

        let body_lines: Vec<&str> = lines
            .iter()
            .skip(line_idx + 1)
            .take(next_heading_line - line_idx - 1)
            .copied()
            .collect();

        let body_excerpt = {
            let full: String = body_lines.join("\n").trim().to_string();
            if full.is_empty() {
                None
            } else {
                Some(full.chars().take(512).collect::<String>())
            }
        };

        let search_text = format!("{} {}", heading_text, body_excerpt.as_deref().unwrap_or(""))
            .trim()
            .to_string();

        result.push(ParsedSection {
            ordinal: ordinal + 1,
            heading_text: heading_text.clone(),
            heading_level: *level,
            anchor_slug: anchor,
            body_excerpt,
            search_text,
        });
    }

    result
}

/// Parse a markdown heading line. Returns (level, text) or None.
fn parse_heading(line: &str) -> Option<(usize, String)> {
    let trimmed = line.trim_start();
    if !trimmed.starts_with('#') {
        return None;
    }
    let level = trimmed.chars().take_while(|c| *c == '#').count();
    if level > 6 {
        return None;
    }
    let text = trimmed[level..].trim().to_string();
    if text.is_empty() {
        return None;
    }
    Some((level, text))
}

/// Generate a stable anchor slug from heading text.
///
/// Rules: lowercase, replace non-alphanumeric with `-`, collapse multiple dashes,
/// strip leading/trailing dashes.
pub fn slugify(text: &str) -> String {
    let mut slug = String::new();
    let mut prev_dash = false;
    for ch in text.chars() {
        if ch.is_alphanumeric() {
            slug.push(ch.to_lowercase().next().unwrap());
            prev_dash = false;
        } else if !prev_dash && !slug.is_empty() {
            slug.push('-');
            prev_dash = true;
        }
    }
    slug.trim_end_matches('-').to_string()
}

// =============================================================================
// Subtype allow-list (mirrors artifact_store.rs — single authoritative copy in parser)
// =============================================================================

/// Validate a subtype against the DOCUMENT_TEMPLATES.md §11.1 allow-list.
pub fn validate_subtype(template: &str, subtype: &str) -> bool {
    match template {
        "T1AuthorityDoc" => matches!(
            subtype,
            "prd"
                | "ux_spec"
                | "interaction_spec"
                | "acceptance_spec"
                | "architecture_design"
                | "architecture_decisions"
        ),
        "T2RoleProfile" => subtype == "seat_role",
        "T3TaskPacket" => matches!(subtype, "task" | "fix" | "integration" | "verification"),
        "T4Review" => matches!(
            subtype,
            "gap_review" | "benchmark" | "process_mapping" | "design_proposal"
        ),
        "T5Acceptance" => matches!(subtype, "acceptance_review" | "gate_decision"),
        "T6DailyMemory" => subtype == "daily_log",
        "T7GovernanceDoc" => matches!(
            subtype,
            "coordination_rules"
                | "workflow_principles"
                | "collaboration_protocol"
                | "document_templates"
        ),
        _ => false,
    }
}

/// Normalize a template alias from document headers into the canonical DB family.
pub fn normalize_template(template: &str) -> Option<&'static str> {
    match template.trim() {
        "T1" | "T1AuthorityDoc" => Some("T1AuthorityDoc"),
        "T2" | "T2RoleProfile" => Some("T2RoleProfile"),
        "T3" | "T3TaskPacket" => Some("T3TaskPacket"),
        "T4" | "T4Review" => Some("T4Review"),
        "T5" | "T5Acceptance" => Some("T5Acceptance"),
        "T6" | "T6DailyMemory" => Some("T6DailyMemory"),
        "T7" | "T7GovernanceDoc" => Some("T7GovernanceDoc"),
        _ => None,
    }
}

/// Validate a document's template and subtype together.
pub fn validate_header_subtype(header: &ParsedDocHeader) -> bool {
    match (&header.template, &header.subtype) {
        (Some(t), Some(s)) => validate_subtype(t, s),
        _ => false,
    }
}

/// Derive the repo-relative file path suffix from a full path.
pub fn repo_relative_path(repo_root: &Path, file_path: &Path) -> String {
    file_path
        .strip_prefix(repo_root)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| file_path.to_string_lossy().into_owned())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_DOC: &str = r#"# SeatLoom PRD v0.5

| Field | Value |
|-------|-------|
| template | T1AuthorityDoc |
| subtype | prd |
| id | prd-v0.5 |
| status | draft |
| author | lyra |
| date | 2026-04-28 |
| version | v0.5 |
| depends_on | `prd-v0.4` |
| supersedes | prd-v0.4 |
| tags | product, requirements, contract |

## 0. Why v0.5 exists

This revision turns the Apr 28 core-value deep dive into an executable product contract.

## 1. Product thesis

SeatLoom is a local-first continuity system for human-plus-agent project work.

### 1.1 Core value

Higher output per cost through deterministic routing and reusable seat systems.
"#;

    #[test]
    fn parses_template_and_subtype() {
        let header = parse_header(SAMPLE_DOC).expect("header must parse");
        assert_eq!(header.template.as_deref(), Some("T1AuthorityDoc"));
        assert_eq!(header.subtype.as_deref(), Some("prd"));
    }

    #[test]
    fn normalizes_short_template_aliases() {
        let doc = r#"# Example

| Field | Value |
|-------|-------|
| template | T3 |
| subtype | task |
| id | example |
"#;

        let header = parse_header(doc).expect("header must parse");
        assert_eq!(header.template.as_deref(), Some("T3TaskPacket"));
        assert_eq!(header.subtype.as_deref(), Some("task"));
    }

    #[test]
    fn ignores_header_examples_inside_code_fences() {
        let doc = r#"# SeatLoom Document Templates v1.0

| Item | Content |
|------|---------|
| Document | Document Templates Specification |

```markdown
# <Document Title>

| Field | Value |
|-------|-------|
| template | T1/T2/T3/T4/T5/T6/T7 |
| subtype | <template-specific subtype> |
```
"#;

        let header = parse_header(doc).expect("header must parse");
        assert_eq!(
            header.title.as_deref(),
            Some("SeatLoom Document Templates v1.0")
        );
        assert_eq!(header.template, None);
        assert_eq!(header.subtype, None);
    }

    #[test]
    fn parses_doc_id_and_status() {
        let header = parse_header(SAMPLE_DOC).expect("header must parse");
        assert_eq!(header.doc_id.as_deref(), Some("prd-v0.5"));
        assert_eq!(header.status.as_deref(), Some("draft"));
    }

    #[test]
    fn parses_author_date_version() {
        let header = parse_header(SAMPLE_DOC).expect("header must parse");
        assert_eq!(header.author.as_deref(), Some("lyra"));
        assert_eq!(header.date.as_deref(), Some("2026-04-28"));
        assert_eq!(header.version.as_deref(), Some("v0.5"));
    }

    #[test]
    fn parses_depends_on_list() {
        let header = parse_header(SAMPLE_DOC).expect("header must parse");
        assert!(!header.depends_on.is_empty());
        assert!(header.depends_on.iter().any(|d| d.contains("prd-v0.4")));
    }

    #[test]
    fn parses_supersedes_and_tags() {
        let header = parse_header(SAMPLE_DOC).expect("header must parse");
        assert_eq!(header.supersedes.as_deref(), Some("prd-v0.4"));
        assert!(!header.tags.is_empty());
        assert!(header.tags.iter().any(|t| t == "product"));
    }

    #[test]
    fn extracts_title_from_h1() {
        let header = parse_header(SAMPLE_DOC).expect("header must parse");
        assert_eq!(header.title.as_deref(), Some("SeatLoom PRD v0.5"));
    }

    #[test]
    fn extracts_sections_with_levels() {
        let sections = extract_sections(SAMPLE_DOC);
        assert!(sections.len() >= 3, "expected at least 3 sections");
        let h2: Vec<_> = sections.iter().filter(|s| s.heading_level == 2).collect();
        assert!(!h2.is_empty());
    }

    #[test]
    fn extracts_section_bodies() {
        let sections = extract_sections(SAMPLE_DOC);
        let thesis = sections
            .iter()
            .find(|s| s.heading_text.contains("Product thesis"));
        assert!(thesis.is_some());
        let thesis = thesis.unwrap();
        assert!(thesis
            .body_excerpt
            .as_deref()
            .unwrap_or("")
            .contains("local-first"));
    }

    #[test]
    fn slugify_produces_lowercase_dashes() {
        assert_eq!(slugify("Product thesis"), "product-thesis");
        assert_eq!(slugify("AD-011: Retrieval Order"), "ad-011-retrieval-order");
        assert_eq!(slugify("1.1 Core value"), "1-1-core-value");
    }

    #[test]
    fn slugify_strips_leading_trailing_dashes() {
        assert_eq!(slugify("  test  "), "test");
        assert_eq!(slugify("## heading"), "heading");
    }

    #[test]
    fn validates_subtype_t1_prd() {
        assert!(validate_subtype("T1AuthorityDoc", "prd"));
        assert!(validate_subtype("T1AuthorityDoc", "ux_spec"));
        assert!(!validate_subtype("T1AuthorityDoc", "task"));
    }

    #[test]
    fn validates_subtype_t3_task() {
        assert!(validate_subtype("T3TaskPacket", "task"));
        assert!(validate_subtype("T3TaskPacket", "fix"));
        assert!(!validate_subtype("T3TaskPacket", "prd"));
    }

    #[test]
    fn validates_all_template_families() {
        assert!(validate_subtype("T2RoleProfile", "seat_role"));
        assert!(validate_subtype("T4Review", "process_mapping"));
        assert!(validate_subtype("T5Acceptance", "acceptance_review"));
        assert!(validate_subtype("T6DailyMemory", "daily_log"));
        assert!(validate_subtype("T7GovernanceDoc", "coordination_rules"));
    }

    #[test]
    fn header_subtype_valid_for_real_doc() {
        let header = parse_header(SAMPLE_DOC).expect("header must parse");
        assert!(validate_header_subtype(&header));
    }

    #[test]
    fn returns_none_for_doc_without_header() {
        let no_header = "# Just a title\n\nSome text without a table.\n";
        // Should still parse (returns empty header with title)
        let header = parse_header(no_header);
        assert!(header.is_some());
        let header = header.unwrap();
        assert_eq!(header.title.as_deref(), Some("Just a title"));
        assert!(header.template.is_none());
    }

    #[test]
    fn sections_have_deterministic_ordinals() {
        let sections = extract_sections(SAMPLE_DOC);
        for (i, s) in sections.iter().enumerate() {
            assert_eq!(s.ordinal, i + 1);
        }
    }
}
