use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root must resolve")
}

fn read(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
}

fn quoted_fields(line: &str) -> Vec<&str> {
    line.split('\'').skip(1).step_by(2).collect()
}

fn baseline_artifact_ids(seed_sql: &str) -> BTreeSet<String> {
    seed_sql
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            if !trimmed.starts_with("('ar-") {
                return None;
            }
            quoted_fields(trimmed).first().map(|id| (*id).to_string())
        })
        .collect()
}

fn document_artifact_refs(seed_sql: &str) -> BTreeSet<String> {
    seed_sql
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            if !trimmed.starts_with("('doc-") {
                return None;
            }
            let fields = quoted_fields(trimmed);
            fields.get(2).map(|id| (*id).to_string())
        })
        .collect()
}

fn document_ids(seed_sql: &str) -> BTreeSet<String> {
    seed_sql
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            if !trimmed.starts_with("('doc-") {
                return None;
            }
            quoted_fields(trimmed).first().map(|id| (*id).to_string())
        })
        .collect()
}

fn ingest_targets(script: &str) -> Vec<(String, String)> {
    script
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if !trimmed.starts_with("ingest_doc '") {
                return None;
            }
            let fields = quoted_fields(trimmed);
            if fields.len() < 2 {
                return None;
            }
            Some((fields[0].to_string(), fields[1].to_string()))
        })
        .collect()
}

#[test]
fn postgres_seed_consistency_document_artifact_refs_exist_in_baseline() {
    let root = workspace_root();
    let baseline_seed = read(&root.join("infra/postgres/seed/001_real_collaboration_baseline.sql"));
    let document_seed = read(&root.join("infra/postgres/seed/002_document_seed.sql"));

    let artifact_ids = baseline_artifact_ids(&baseline_seed);
    let document_refs = document_artifact_refs(&document_seed);
    let missing: Vec<_> = document_refs.difference(&artifact_ids).cloned().collect();

    assert!(
        missing.is_empty(),
        "document seed references artifact ids missing from baseline seed: {missing:?}"
    );
}

#[test]
fn postgres_seed_consistency_ingest_doc_ids_match_seeded_documents() {
    let root = workspace_root();
    let document_seed = read(&root.join("infra/postgres/seed/002_document_seed.sql"));
    let ingest_script = read(&root.join("scripts/ingest-documents.sh"));

    let seeded_doc_ids = document_ids(&document_seed);
    let ingested_doc_ids: BTreeSet<_> = ingest_targets(&ingest_script)
        .into_iter()
        .map(|(doc_id, _)| doc_id)
        .collect();

    assert_eq!(
        ingested_doc_ids, seeded_doc_ids,
        "ingest-documents.sh doc ids must match 002_document_seed.sql"
    );
}

#[test]
fn postgres_seed_consistency_ingest_paths_exist_in_repo() {
    let root = workspace_root();
    let ingest_script = read(&root.join("scripts/ingest-documents.sh"));

    for (doc_id, file_path) in ingest_targets(&ingest_script) {
        let full_path = root.join(&file_path);
        assert!(
            full_path.is_file(),
            "ingest target for {doc_id} is missing: {}",
            full_path.display()
        );
    }
}
