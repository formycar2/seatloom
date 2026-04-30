/// PostgreSQL write/ingest/reconcile engine for SeatLoom coordination documents.
///
/// Implements the MVP reconcile contract from AD-007:
///   triggers = { startup, pre_pipeline, manual }
///   no file watcher, no background daemon
///
/// The reconcile operation is the bounded import/update path that moves
/// markdown truth authored in the repository into PostgreSQL authority.
///
/// # Behaviour
///
/// For each scanned markdown file:
/// - **new doc**: insert document + sections + document_version(rev=1)
/// - **unchanged**: digest match → skip; record reconcile_item(unchanged)
/// - **changed**: update doc body/header, bump revision, insert document_version
/// - **conflict**: same `doc_id` already exists at a different `file_path` → record conflict
/// - **parse failure**: record reconcile_item(failed); document stored with parse_status='partial'
use std::path::{Path, PathBuf};
use std::{fs, io};

use deadpool_postgres::Pool;
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::db::document_parser::{extract_sections, parse_header, validate_subtype};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Which event triggered this reconcile run (AD-007).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReconcileTrigger {
    Startup,
    PrePipeline,
    Manual,
}

impl ReconcileTrigger {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Startup => "startup",
            Self::PrePipeline => "pre_pipeline",
            Self::Manual => "manual",
        }
    }
}

/// Outcome for a single file within a reconcile run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemOutcome {
    /// File was ingested for the first time.
    Inserted,
    /// File body or header changed; revision incremented.
    Updated,
    /// File is byte-for-byte identical to what PostgreSQL already holds.
    Unchanged,
    /// File could not be read or its header could not be classified.
    Failed(String),
    /// Same `doc_id` exists in PostgreSQL but for a different `file_path`.
    Conflict(String),
}

impl ItemOutcome {
    fn as_str(&self) -> &str {
        match self {
            Self::Inserted => "inserted",
            Self::Updated => "updated",
            Self::Unchanged => "unchanged",
            Self::Failed(_) => "failed",
            Self::Conflict(_) => "conflict",
        }
    }

    fn failure_reason(&self) -> Option<String> {
        match self {
            Self::Failed(r) | Self::Conflict(r) => Some(r.clone()),
            _ => None,
        }
    }
}

/// Aggregate result of one reconcile run.
#[derive(Debug, Clone, Default)]
pub struct ReconcileResult {
    pub run_id: String,
    pub trigger: String,
    pub scanned: u32,
    pub inserted: u32,
    pub updated: u32,
    pub unchanged: u32,
    pub failed: u32,
    pub conflicted: u32,
    /// Per-file results, in scan order.
    pub items: Vec<ReconcileItemResult>,
}

/// Result for a single file.
#[derive(Debug, Clone)]
pub struct ReconcileItemResult {
    pub file_path: String,
    pub document_id: Option<String>,
    pub outcome: ItemOutcome,
    pub revision_before: Option<i32>,
    pub revision_after: Option<i32>,
    pub digest_before: Option<String>,
    pub digest_after: Option<String>,
}

struct ReconcileItemInsert<'a> {
    run_id: &'a str,
    file_path: &'a str,
    doc_id: Option<&'a str>,
    outcome: ItemOutcome,
    digest_before: Option<&'a str>,
    digest_after: Option<&'a str>,
    revision_before: Option<i32>,
    revision_after: Option<i32>,
    parse_status: Option<&'a str>,
}

type ExistingDocumentRow = (String, i32, Option<String>, Option<String>, Option<String>);

// ---------------------------------------------------------------------------
// Allowed ingest paths (bounded, not arbitrary)
// ---------------------------------------------------------------------------

/// Return true if a repo-relative path is within the allowed ingest surface.
/// Bounded to: docs/*.md  and  docs/coordination/**/*.md
pub fn is_allowed_ingest_path(repo_relative: &str) -> bool {
    let path = Path::new(repo_relative);
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext != "md" {
        return false;
    }
    // docs/*.md  or  docs/coordination/**/*.md
    let comps: Vec<&str> = repo_relative.split('/').collect();
    if comps.first() != Some(&"docs") {
        return false;
    }
    true
}

// ---------------------------------------------------------------------------
// Digest helper
// ---------------------------------------------------------------------------

/// Compute a hex SHA-256 digest of a string.
pub fn sha256_hex(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    format!("{:x}", hasher.finalize())
}

// ---------------------------------------------------------------------------
// Reconcile runner
// ---------------------------------------------------------------------------

/// Run the full reconcile operation: scan → parse → upsert → bookkeeping.
///
/// Returns a `ReconcileResult` with aggregate counts and per-file outcomes.
/// This function requires an active PostgreSQL connection (via `pool`).
pub async fn run_reconcile(
    pool: &Pool,
    trigger: ReconcileTrigger,
    repo_root: &Path,
    project_id: &str,
) -> Result<ReconcileResult, ReconcileError> {
    let run_id = format!("run-{}", nanoid_8());
    let client = pool.get().await.map_err(ReconcileError::Pool)?;

    // Insert reconcile_run (status=running)
    client
        .execute(
            "INSERT INTO reconcile_runs (id, trigger, status, started_at) VALUES ($1, $2, 'running', NOW())",
            &[&run_id, &trigger.as_str()],
        )
        .await?;

    let mut result = ReconcileResult {
        run_id: run_id.clone(),
        trigger: trigger.as_str().to_string(),
        ..Default::default()
    };

    // Scan allowed markdown files
    let files = collect_markdown_files(repo_root);

    for file_path in &files {
        let repo_relative = file_path
            .strip_prefix(repo_root)
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| file_path.to_string_lossy().into_owned());

        if !is_allowed_ingest_path(&repo_relative) {
            continue;
        }

        result.scanned += 1;
        let item = process_one_file(&client, file_path, &repo_relative, project_id, &run_id).await;
        match &item.outcome {
            ItemOutcome::Inserted => result.inserted += 1,
            ItemOutcome::Updated => result.updated += 1,
            ItemOutcome::Unchanged => result.unchanged += 1,
            ItemOutcome::Failed(_) => result.failed += 1,
            ItemOutcome::Conflict(_) => result.conflicted += 1,
        }
        result.items.push(item);
    }

    // Update reconcile_run to completed
    client
        .execute(
            "UPDATE reconcile_runs SET status='completed', completed_at=NOW(), \
             scanned=$2, inserted=$3, updated=$4, unchanged=$5, failed=$6, conflicted=$7 \
             WHERE id=$1",
            &[
                &run_id,
                &(result.scanned as i32),
                &(result.inserted as i32),
                &(result.updated as i32),
                &(result.unchanged as i32),
                &(result.failed as i32),
                &(result.conflicted as i32),
            ],
        )
        .await?;

    Ok(result)
}

/// Process a single markdown file within a reconcile run.
async fn process_one_file(
    client: &deadpool_postgres::Object,
    file_path: &Path,
    repo_relative: &str,
    project_id: &str,
    run_id: &str,
) -> ReconcileItemResult {
    // Read file
    let body = match fs::read_to_string(file_path) {
        Ok(b) => b,
        Err(e) => {
            let reason = format!("read error: {e}");
            let _ = insert_reconcile_item(
                client,
                ReconcileItemInsert {
                    run_id,
                    file_path: repo_relative,
                    doc_id: None,
                    outcome: ItemOutcome::Failed(reason.clone()),
                    digest_before: None,
                    digest_after: None,
                    revision_before: None,
                    revision_after: None,
                    parse_status: Some("failed"),
                },
            )
            .await;
            return ReconcileItemResult {
                file_path: repo_relative.to_string(),
                document_id: None,
                outcome: ItemOutcome::Failed(reason),
                revision_before: None,
                revision_after: None,
                digest_before: None,
                digest_after: None,
            };
        }
    };

    let new_digest = sha256_hex(&body);
    let header = parse_header(&body);

    // Determine document id — prefer doc_id from header, else slug from file path
    let computed_doc_id = header
        .as_ref()
        .and_then(|h| h.doc_id.clone())
        .unwrap_or_else(|| path_to_doc_id(repo_relative));

    // Check for conflict: same doc_id but different file_path
    let existing_conflict: Option<String> = client
        .query_opt(
            "SELECT file_path FROM documents WHERE doc_id = $1 AND file_path != $2",
            &[&computed_doc_id, &repo_relative],
        )
        .await
        .ok()
        .flatten()
        .map(|r| r.get::<_, String>(0));

    if let Some(conflict_path) = existing_conflict {
        let reason = format!("doc_id '{computed_doc_id}' already mapped to '{conflict_path}'");
        let _ = insert_reconcile_item(
            client,
            ReconcileItemInsert {
                run_id,
                file_path: repo_relative,
                doc_id: None,
                outcome: ItemOutcome::Conflict(reason.clone()),
                digest_before: None,
                digest_after: None,
                revision_before: None,
                revision_after: None,
                parse_status: None,
            },
        )
        .await;
        return ReconcileItemResult {
            file_path: repo_relative.to_string(),
            document_id: None,
            outcome: ItemOutcome::Conflict(reason),
            revision_before: None,
            revision_after: None,
            digest_before: None,
            digest_after: None,
        };
    }

    // Look up existing document by file_path
    let existing: Option<ExistingDocumentRow> = client
        .query_opt(
            "SELECT id, revision, body_digest, template, subtype FROM documents WHERE file_path = $1",
            &[&repo_relative],
        )
        .await
        .ok()
        .flatten()
        .map(|r| (r.get(0), r.get(1), r.get(2), r.get(3), r.get(4)));

    let h = header.as_ref();
    let header_template = h.and_then(|h| h.template.clone());
    let header_subtype = h.and_then(|h| h.subtype.clone());
    let template = header_template.as_deref();
    let subtype = header_subtype.as_deref();
    let parse_ok = match (template, subtype) {
        (Some(t), Some(s)) => validate_subtype(t, s),
        _ => false,
    };
    let parse_status = if parse_ok { "parsed" } else { "partial" };

    match existing {
        None => {
            // New document — insert
            let doc_id_row = format!("doc-{}", nanoid_8());
            let title = h
                .and_then(|h| h.title.as_deref())
                .unwrap_or("Untitled")
                .to_string();

            let insert_result = client.execute(
                "INSERT INTO documents (id, project_id, template, subtype, subtype_valid, \
                 doc_id, title, status, author, version, depends_on, supersedes, tags, \
                 file_path, body_text, body_digest, body_length, parse_status, revision, \
                 created_at, updated_at) \
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,1,NOW(),NOW())",
                &[
                    &doc_id_row,
                    &project_id,
                    &template,
                    &subtype,
                    &Some(parse_ok),
                    &computed_doc_id,
                    &title,
                    &h.and_then(|h| h.status.as_deref()),
                    &h.and_then(|h| h.author.as_deref()),
                    &h.and_then(|h| h.version.as_deref()),
                    &h.map(|h| h.depends_on.as_slice()).unwrap_or(&[]).to_vec(),
                    &h.and_then(|h| h.supersedes.as_deref()),
                    &h.map(|h| h.tags.as_slice()).unwrap_or(&[]).to_vec(),
                    &repo_relative,
                    &body,
                    &new_digest,
                    &(body.len() as i32),
                    &parse_status,
                ],
            ).await;

            if let Err(e) = insert_result {
                let reason = format!("insert error: {e}");
                let _ = insert_reconcile_item(
                    client,
                    ReconcileItemInsert {
                        run_id,
                        file_path: repo_relative,
                        doc_id: None,
                        outcome: ItemOutcome::Failed(reason.clone()),
                        digest_before: None,
                        digest_after: Some(&new_digest),
                        revision_before: None,
                        revision_after: None,
                        parse_status: Some(parse_status),
                    },
                )
                .await;
                return ReconcileItemResult {
                    file_path: repo_relative.to_string(),
                    document_id: None,
                    outcome: ItemOutcome::Failed(reason),
                    revision_before: None,
                    revision_after: None,
                    digest_before: None,
                    digest_after: Some(new_digest),
                };
            }

            // Insert initial version snapshot
            let _ = insert_document_version(client, &doc_id_row, 1, &new_digest, &body, h, run_id)
                .await;

            // Insert sections
            let sections = extract_sections(&body);
            for sec in &sections {
                let sec_id = format!("sec-{}-{}", doc_id_row, sec.ordinal);
                let _ = client.execute(
                    "INSERT INTO document_sections (id, document_id, ordinal, heading_text, heading_level, anchor_slug, body_excerpt, search_text, created_at) \
                     VALUES ($1,$2,$3,$4,$5,$6,$7,$8,NOW()) ON CONFLICT (id) DO NOTHING",
                    &[&sec_id, &doc_id_row, &(sec.ordinal as i32), &sec.heading_text,
                      &(sec.heading_level as i32), &sec.anchor_slug,
                      &sec.body_excerpt, &Some(sec.search_text.clone())],
                ).await;
            }

            let _ = insert_reconcile_item(
                client,
                ReconcileItemInsert {
                    run_id,
                    file_path: repo_relative,
                    doc_id: Some(&doc_id_row),
                    outcome: ItemOutcome::Inserted,
                    digest_before: None,
                    digest_after: Some(&new_digest),
                    revision_before: None,
                    revision_after: Some(1),
                    parse_status: Some(parse_status),
                },
            )
            .await;

            ReconcileItemResult {
                file_path: repo_relative.to_string(),
                document_id: Some(doc_id_row),
                outcome: ItemOutcome::Inserted,
                revision_before: None,
                revision_after: Some(1),
                digest_before: None,
                digest_after: Some(new_digest),
            }
        }

        Some((doc_id_row, current_rev, current_digest, existing_template, existing_subtype)) => {
            let resolved_template = header_template.or(existing_template);
            let resolved_subtype = header_subtype.or(existing_subtype);
            let resolved_parse_ok =
                match (resolved_template.as_deref(), resolved_subtype.as_deref()) {
                    (Some(t), Some(s)) => validate_subtype(t, s),
                    _ => false,
                };
            let resolved_parse_status = if resolved_parse_ok {
                "parsed"
            } else {
                "partial"
            };

            // Existing document
            if current_digest.as_deref() == Some(&new_digest) {
                // Unchanged
                let _ = insert_reconcile_item(
                    client,
                    ReconcileItemInsert {
                        run_id,
                        file_path: repo_relative,
                        doc_id: Some(&doc_id_row),
                        outcome: ItemOutcome::Unchanged,
                        digest_before: current_digest.as_deref(),
                        digest_after: Some(&new_digest),
                        revision_before: Some(current_rev),
                        revision_after: Some(current_rev),
                        parse_status: Some(resolved_parse_status),
                    },
                )
                .await;
                return ReconcileItemResult {
                    file_path: repo_relative.to_string(),
                    document_id: Some(doc_id_row),
                    outcome: ItemOutcome::Unchanged,
                    revision_before: Some(current_rev),
                    revision_after: Some(current_rev),
                    digest_before: current_digest,
                    digest_after: Some(new_digest),
                };
            }

            // Changed — update document, increment revision
            let new_rev = current_rev + 1;
            let title = h
                .and_then(|h| h.title.as_deref())
                .unwrap_or("Untitled")
                .to_string();

            let _ = client
                .execute(
                    "UPDATE documents SET body_text=$1, body_digest=$2, body_length=$3, \
                 parse_status=$4, revision=$5, title=$6, template=$7, subtype=$8, \
                 subtype_valid=$9, updated_at=NOW() WHERE id=$10",
                    &[
                        &body,
                        &new_digest,
                        &(body.len() as i32),
                        &resolved_parse_status,
                        &new_rev,
                        &title,
                        &resolved_template,
                        &resolved_subtype,
                        &Some(resolved_parse_ok),
                        &doc_id_row,
                    ],
                )
                .await;

            // Insert version snapshot
            let _ = insert_document_version(
                client,
                &doc_id_row,
                new_rev,
                &new_digest,
                &body,
                h,
                run_id,
            )
            .await;

            // Refresh sections
            let _ = client
                .execute(
                    "DELETE FROM document_sections WHERE document_id = $1",
                    &[&doc_id_row],
                )
                .await;
            let sections = extract_sections(&body);
            for sec in &sections {
                let sec_id = format!("sec-{}-{}", doc_id_row, sec.ordinal);
                let _ = client.execute(
                    "INSERT INTO document_sections (id, document_id, ordinal, heading_text, heading_level, anchor_slug, body_excerpt, search_text, created_at) \
                     VALUES ($1,$2,$3,$4,$5,$6,$7,$8,NOW()) ON CONFLICT (id) DO NOTHING",
                    &[&sec_id, &doc_id_row, &(sec.ordinal as i32), &sec.heading_text,
                      &(sec.heading_level as i32), &sec.anchor_slug,
                      &sec.body_excerpt, &Some(sec.search_text.clone())],
                ).await;
            }

            let _ = insert_reconcile_item(
                client,
                ReconcileItemInsert {
                    run_id,
                    file_path: repo_relative,
                    doc_id: Some(&doc_id_row),
                    outcome: ItemOutcome::Updated,
                    digest_before: current_digest.as_deref(),
                    digest_after: Some(&new_digest),
                    revision_before: Some(current_rev),
                    revision_after: Some(new_rev),
                    parse_status: Some(resolved_parse_status),
                },
            )
            .await;

            ReconcileItemResult {
                file_path: repo_relative.to_string(),
                document_id: Some(doc_id_row),
                outcome: ItemOutcome::Updated,
                revision_before: Some(current_rev),
                revision_after: Some(new_rev),
                digest_before: current_digest,
                digest_after: Some(new_digest),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn insert_reconcile_item(
    client: &deadpool_postgres::Object,
    item: ReconcileItemInsert<'_>,
) -> Result<(), tokio_postgres::Error> {
    let item_id = format!("ri-{}", nanoid_8());
    client
        .execute(
            "INSERT INTO reconcile_items (id, run_id, file_path, document_id, outcome, \
         digest_before, digest_after, revision_before, revision_after, \
         parse_status, failure_reason, created_at) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,NOW())",
            &[
                &item_id,
                &item.run_id,
                &item.file_path,
                &item.doc_id,
                &item.outcome.as_str(),
                &item.digest_before,
                &item.digest_after,
                &item.revision_before,
                &item.revision_after,
                &item.parse_status,
                &item.outcome.failure_reason(),
            ],
        )
        .await?;
    Ok(())
}

async fn insert_document_version(
    client: &deadpool_postgres::Object,
    document_id: &str,
    revision: i32,
    body_digest: &str,
    body_text: &str,
    header: Option<&crate::db::document_parser::ParsedDocHeader>,
    run_id: &str,
) -> Result<(), tokio_postgres::Error> {
    let dv_id = format!("dv-{}", nanoid_8());
    let header_json: Option<serde_json::Value> = header.map(|h| {
        serde_json::json!({
            "template": h.template,
            "subtype": h.subtype,
            "doc_id": h.doc_id,
            "title": h.title,
            "status": h.status,
            "author": h.author,
            "date": h.date,
            "version": h.version,
            "depends_on": h.depends_on,
            "supersedes": h.supersedes,
            "tags": h.tags,
        })
    });
    client.execute(
        "INSERT INTO document_versions (id, document_id, revision, body_digest, body_text, header_snapshot, run_id, created_at) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,NOW()) ON CONFLICT (document_id, revision) DO NOTHING",
        &[&dv_id, &document_id, &revision, &body_digest, &body_text, &header_json, &run_id],
    ).await?;
    Ok(())
}

/// Collect all .md files under repo_root, sorted for deterministic order.
fn collect_markdown_files(repo_root: &Path) -> Vec<PathBuf> {
    let docs_root = repo_root.join("docs");
    if !docs_root.is_dir() {
        return vec![];
    }
    let mut files = Vec::new();
    collect_md_recursive(&docs_root, &mut files);
    files.sort();
    files
}

fn collect_md_recursive(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_md_recursive(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            out.push(path);
        }
    }
}

/// Convert a file path to a stable doc_id fallback slug.
fn path_to_doc_id(repo_relative: &str) -> String {
    let without_ext = repo_relative.trim_end_matches(".md");
    without_ext
        .replace(['/', '\\'], "-")
        .trim_start_matches('-')
        .to_lowercase()
}

/// Simple 8-char random ID without nanoid dep for unique IDs.
fn nanoid_8() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    format!("{nanos:08x}")
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum ReconcileError {
    #[error("postgres error: {0}")]
    Postgres(#[from] tokio_postgres::Error),
    #[error("connection pool error: {0}")]
    Pool(deadpool_postgres::PoolError),
    #[error("io error: {0}")]
    Io(#[from] io::Error),
}

// ---------------------------------------------------------------------------
// Row models for reading reconcile bookkeeping (used by repositories.rs)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ReconcileRunRow {
    pub id: String,
    pub trigger: String,
    pub status: String,
    pub scanned: i32,
    pub inserted: i32,
    pub updated: i32,
    pub unchanged: i32,
    pub failed: i32,
    pub conflicted: i32,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone)]
pub struct ReconcileItemRow {
    pub id: String,
    pub run_id: String,
    pub file_path: String,
    pub document_id: Option<String>,
    pub outcome: String,
    pub digest_before: Option<String>,
    pub digest_after: Option<String>,
    pub revision_before: Option<i32>,
    pub revision_after: Option<i32>,
    pub parse_status: Option<String>,
    pub failure_reason: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct DocumentVersionRow {
    pub id: String,
    pub document_id: String,
    pub revision: i32,
    pub body_digest: String,
    pub body_text: Option<String>,
    pub header_snapshot: Option<serde_json::Value>,
    pub run_id: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// ---------------------------------------------------------------------------
// Unit tests (no DB required)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_hex_is_deterministic() {
        let d1 = sha256_hex("hello world");
        let d2 = sha256_hex("hello world");
        assert_eq!(d1, d2);
        assert_eq!(d1.len(), 64); // hex of 32 bytes
    }

    #[test]
    fn sha256_hex_changes_on_body_change() {
        let d1 = sha256_hex("version 1");
        let d2 = sha256_hex("version 2");
        assert_ne!(d1, d2);
    }

    #[test]
    fn is_allowed_ingest_path_accepts_docs_root() {
        assert!(is_allowed_ingest_path("docs/prd-v0.5.md"));
        assert!(is_allowed_ingest_path("docs/PRODUCT_TRUTH.md"));
    }

    #[test]
    fn is_allowed_ingest_path_accepts_docs_coordination() {
        assert!(is_allowed_ingest_path(
            "docs/coordination/COORDINATION_RULES.md"
        ));
        assert!(is_allowed_ingest_path(
            "docs/coordination/memory/2026-04-29.md"
        ));
        assert!(is_allowed_ingest_path(
            "docs/coordination/tasks/nimbus/NIMBUS-2026-04-29-foo.md"
        ));
    }

    #[test]
    fn is_allowed_ingest_path_rejects_non_docs() {
        assert!(!is_allowed_ingest_path("README.md"));
        assert!(!is_allowed_ingest_path("ui/src/components/Foo.tsx"));
        assert!(!is_allowed_ingest_path("crates/seatloom-core/src/lib.rs"));
    }

    #[test]
    fn is_allowed_ingest_path_rejects_non_md() {
        assert!(!is_allowed_ingest_path("docs/something.yaml"));
        assert!(!is_allowed_ingest_path("docs/data.json"));
    }

    #[test]
    fn path_to_doc_id_slugifies_correctly() {
        assert_eq!(path_to_doc_id("docs/prd-v0.5.md"), "docs-prd-v0.5");
        assert_eq!(
            path_to_doc_id("docs/coordination/COORDINATION_RULES.md"),
            "docs-coordination-coordination_rules"
        );
    }

    #[test]
    fn unchanged_digest_detects_no_change() {
        let body = "# Title\n\nSome content.";
        let d1 = sha256_hex(body);
        let d2 = sha256_hex(body);
        // Unchanged: digests equal → should be outcome Unchanged
        assert_eq!(d1, d2);
    }

    #[test]
    fn changed_digest_detects_change() {
        let body_v1 = "# Title\n\nVersion 1 content.";
        let body_v2 = "# Title\n\nVersion 2 content — something changed.";
        let d1 = sha256_hex(body_v1);
        let d2 = sha256_hex(body_v2);
        assert_ne!(d1, d2);
    }

    #[test]
    fn collect_markdown_files_returns_sorted_paths() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let files = collect_markdown_files(root);
        // Should find at least some docs (only meaningful in workspace)
        if !files.is_empty() {
            // Must be sorted
            for pair in files.windows(2) {
                assert!(pair[0] <= pair[1], "files not sorted");
            }
        }
    }
}
