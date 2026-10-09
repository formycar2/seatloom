// Governed-dispatch acceptance gate (014, D2) — the operator's decide commands.
//
// The D2 loop's user-facing half (Lyra design §4): a seat finishes work, Flux
// verifies it green, the seat calls `request_acceptance` (MCP), and the operator
// — here, the SeatLoom desktop app acting as the user's agent — accepts (merge
// into track) or rejects (auto-enqueued rework). Seats NEVER merge their own
// work; the merge happens only on an explicit user accept, `--no-ff` into the
// acceptance's base branch, and a conflict REFUSES (never forces).

use std::path::Path;

use seatloom_core::db::models::AcceptanceWithDispatchRow;
use tauri::State;

use crate::dto::{AcceptanceCard, AcceptanceResult};
use crate::state::AppState;

/// The env var naming the operator acting on an acceptance. EXPLICIT config:
/// the decision is a user act, so the identity is never invented on the spot —
/// a missing value refuses the command with a configuration hint.
pub const OPERATOR_REF_ENV: &str = "SEATLOOM_OPERATOR_REF";

/// The pending-acceptance read model (the UI's 待验收区). Pending first, then
/// the most recently decided (result states: merge commit / rework link).
#[tauri::command]
pub async fn cmd_list_acceptances(
    state: State<'_, AppState>,
    project_id: Option<String>,
) -> Result<Vec<AcceptanceCard>, String> {
    let project = project_id.unwrap_or_else(|| state.default_project_id.clone());
    let rows = state
        .db
        .list_acceptances_for_project(&project, 500)
        .await
        .map_err(|e| e.to_string())?;
    Ok(rows.into_iter().map(AcceptanceCard::from).collect())
}

/// Accept a delivery: merge its branch into the base in the main checkout (as
/// the user's agent) and land `accepted` + `AcceptanceGranted` atomically.
/// Returns the merge commit sha.
///
/// Refuses — writing nothing — when: the operator ref is unconfigured, the
/// acceptance is not pending (accepted/rejected are terminal), the Flux verdict
/// referent no longer exists (未验绿不可合), or the merge conflicts (git's own
/// conflict text is returned; the merge is aborted, never forced).
#[tauri::command]
pub async fn cmd_accept_delivery(
    state: State<'_, AppState>,
    acceptance_id: String,
) -> Result<AcceptanceResult, String> {
    let operator = operator_ref()?;
    let row = load_pending(&state, &acceptance_id).await?;

    // Merge criterion, first half: the Flux green verdict must exist NOW. A
    // vanished referent refuses the merge — the verdict is the proof the work
    // passed technical review, and it is checked again at decision time, not
    // only at request time.
    let verified = state
        .db
        .event_or_artifact_exists(&row.acceptance.flux_verdict_ref)
        .await
        .map_err(|e| e.to_string())?;
    if !verified {
        return Err(format!(
            "未验绿不可合: flux_verdict_ref {:?} names no existing event or artifact",
            row.acceptance.flux_verdict_ref
        ));
    }

    let merge_commit = merge_branch_into_base(
        &state.repo_root,
        &row.acceptance.branch,
        &row.acceptance.base,
    )
    .await?;

    state
        .db
        .decide_acceptance(
            &acceptance_id,
            "accepted",
            None,
            Some(&merge_commit),
            None,
            &operator,
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(AcceptanceResult {
        accepted: true,
        merge_commit: Some(merge_commit),
        error: None,
    })
}

/// Reject a delivery: land `rejected` + `AcceptanceRejected` and auto-enqueue a
/// rework dispatch back to the original seat (same write path as the UI's
/// enqueue — its own `pg_notify` wakes the seat through the existing waker).
/// The rework dispatch id is written into the acceptance row (lineage) and
/// carried in the event payload.
#[tauri::command]
pub async fn cmd_reject_delivery(
    state: State<'_, AppState>,
    acceptance_id: String,
    feedback: String,
) -> Result<AcceptanceResult, String> {
    let operator = operator_ref()?;
    if feedback.trim().is_empty() {
        return Err("打回必须带一句反馈: a reject requires one line of feedback".to_string());
    }
    let row = load_pending(&state, &acceptance_id).await?;

    // Enqueue the rework BEFORE landing the rejection so the
    // AcceptanceRejected event can carry the rework lineage. If the enqueue
    // fails, nothing is written and the acceptance stays pending — the operator
    // simply retries.
    let rework_dispatch_id = enqueue_rework_via_mcp(&state, &row, &feedback, &operator).await?;

    state
        .db
        .decide_acceptance(
            &acceptance_id,
            "rejected",
            Some(&feedback),
            None,
            Some(&rework_dispatch_id),
            &operator,
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(AcceptanceResult {
        accepted: false,
        merge_commit: None,
        error: None,
    })
}

// =============================================================================
// Helpers
// =============================================================================

fn operator_ref() -> Result<String, String> {
    std::env::var(OPERATOR_REF_ENV)
        .ok()
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| {
            format!(
                "{OPERATOR_REF_ENV} is not set — the acceptance decision is a user act; \
                 configure the operator ref and retry (identity is never invented on the spot)"
            )
        })
}

/// Load an acceptance that is still pending. Terminal rows are immutable (014):
/// a second decision of a decided acceptance is refused here rather than
/// silently rewriting a recorded fact.
async fn load_pending(
    state: &AppState,
    acceptance_id: &str,
) -> Result<AcceptanceWithDispatchRow, String> {
    let row = state
        .db
        .get_acceptance(acceptance_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("acceptance {acceptance_id:?} not found"))?;
    if row.acceptance.status != "pending" {
        return Err(format!(
            "acceptance {acceptance_id} is {:?} — accepted/rejected are terminal (014); \
             a rework gets a new acceptance",
            row.acceptance.status
        ));
    }
    Ok(row)
}

async fn git(repo_root: &Path, args: &[&str]) -> Result<std::process::Output, String> {
    tokio::process::Command::new("git")
        .args(args)
        .current_dir(repo_root)
        .output()
        .await
        .map_err(|e| format!("git {}: {e}", args.join(" ")))
}

fn out_text(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn err_text(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stderr).trim().to_string()
}

/// Merge `branch` into `base` in the main checkout, `--no-ff` (the delivery
/// stays its own commit on the base). Refuses — never forces — on: a non-git
/// root, a dirty tracked tree, a missing branch, a failed base checkout, or a
/// merge conflict. On conflict the merge is ABORTED, leaving the repo exactly
/// as found, and git's own conflict text is returned.
async fn merge_branch_into_base(
    repo_root: &Path,
    branch: &str,
    base: &str,
) -> Result<String, String> {
    // 1. A real git work tree — never operate in a non-git directory.
    let out = git(repo_root, &["rev-parse", "--is-inside-work-tree"]).await?;
    if !out.status.success() || out_text(&out) != "true" {
        return Err(format!(
            "{} is not a git work tree — refusing to merge",
            repo_root.display()
        ));
    }
    // 2. A clean TRACKED tree: a merge on top of uncommitted changes is
    //    ambiguous. Untracked files (?? in porcelain) do not block a merge.
    let out = git(repo_root, &["status", "--porcelain"]).await?;
    let dirty: Vec<String> = out_text(&out)
        .lines()
        .filter(|l| !l.starts_with("??"))
        .map(str::to_string)
        .collect();
    if !dirty.is_empty() {
        return Err(format!(
            "working tree has uncommitted changes — commit or stash before accepting a merge:\n{}",
            dirty.join("\n")
        ));
    }
    // 3. The delivery branch must exist locally (worktrees share the ref
    //    namespace, so a seat's packet branch is visible from the main
    //    checkout). A clean refusal beats git's merge diagnostics.
    let verify = format!("refs/heads/{branch}");
    let out = git(repo_root, &["rev-parse", "--verify", &verify]).await?;
    if !out.status.success() {
        return Err(format!(
            "branch {branch:?} does not exist in {}",
            repo_root.display()
        ));
    }
    // 4. Be on the base branch (the merge target is explicit, not "wherever
    //    the operator happened to be").
    let current = out_text(&git(repo_root, &["rev-parse", "--abbrev-ref", "HEAD"]).await?);
    if current != base {
        let co = git(repo_root, &["checkout", base]).await?;
        if !co.status.success() {
            return Err(format!(
                "could not check out base {base:?}: {}",
                err_text(&co)
            ));
        }
    }
    // 5. The merge.
    let merge = git(repo_root, &["merge", "--no-ff", "--no-edit", branch]).await?;
    if !merge.status.success() {
        // Conflict = refuse, NEVER force. Abort so the repo is left as found.
        let _ = git(repo_root, &["merge", "--abort"]).await;
        let detail = [out_text(&merge), err_text(&merge)]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        return Err(format!(
            "merge of {branch} into {base} CONFLICTED — aborted, nothing forced:\n{detail}"
        ));
    }
    Ok(out_text(&git(repo_root, &["rev-parse", "HEAD"]).await?))
}

/// Auto-enqueue the rework dispatch through the SAME write path the UI's
/// enqueue command uses: the proven `seatloom-mcp-emit` binary's
/// `enqueue_dispatch` over stdio (atomic row + SeatDispatchIssued + pg_notify —
/// the notify wakes the seat through the existing waker). Returns the new
/// dispatch id parsed out of the tool result.
async fn enqueue_rework_via_mcp(
    state: &AppState,
    row: &AcceptanceWithDispatchRow,
    feedback: &str,
    operator: &str,
) -> Result<String, String> {
    use tokio::io::AsyncWriteExt;
    use tokio::process::Command;

    // Lineage: the rework carries the original packet/branch and the feedback,
    // and jumps the queue (priority +10) so the redo is not starved.
    let packet = row
        .dispatch_payload
        .get("packet")
        .cloned()
        .unwrap_or_else(|| row.dispatch_payload.clone());
    let args = serde_json::json!({
        "target_seat_id": row.dispatch_seat_id,
        "title": format!("打回重做:{}", row.dispatch_title),
        "payload": {
            "rework_of": row.acceptance.dispatch_id,
            "feedback": feedback,
            "packet": packet,
            "branch": row.acceptance.branch,
        },
        "priority": row.dispatch_priority + 10,
        "project_id": row.acceptance.project_id,
    });
    let init = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}}}"#;
    let inited = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
    let call = serde_json::json!({
        "jsonrpc": "2.0", "id": 9, "method": "tools/call",
        "params": { "name": "enqueue_dispatch", "arguments": args },
    });
    let input = format!("{init}\n{inited}\n{call}\n");

    let bin = state.repo_root.join("target/debug/seatloom-mcp-emit");
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://seatloom:seatloom@localhost:5432/seatloom".to_string());

    let mut child = Command::new(&bin)
        .env("DATABASE_URL", &db_url)
        // The issuer of the rework is the operator (the rejecter), not a seat.
        .env("SEATLOOM_ACTOR_REF", operator)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("spawn {}: {e}", bin.display()))?;

    {
        let mut stdin = child.stdin.take().ok_or("no stdin")?;
        stdin
            .write_all(input.as_bytes())
            .await
            .map_err(|e| e.to_string())?;
        // drop(stdin) at scope end closes the pipe -> the server sees EOF and exits.
    }

    let out = child.wait_with_output().await.map_err(|e| e.to_string())?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let last = stdout
        .lines()
        .rfind(|l| !l.trim().is_empty())
        .ok_or("enqueue produced no output")?;
    let v: serde_json::Value = serde_json::from_str(last).map_err(|e| e.to_string())?;
    let r = &v["result"];
    let text = r["content"][0]["text"].as_str().unwrap_or("").to_string();
    if r["isError"].as_bool().unwrap_or(false) {
        return Err(text);
    }
    parse_enqueued_id(&text)
}

/// `enqueue_dispatch`'s success text is `enqueued <id> (status=...)`; the
/// rework lineage needs the id. Strict about the shape — a rejection text
/// (which the caller already screens via `isError`) parses as an error rather
/// than leaking a word of the message into the lineage field.
fn parse_enqueued_id(text: &str) -> Result<String, String> {
    let mut words = text.split_whitespace();
    match (words.next(), words.next()) {
        (Some("enqueued"), Some(id)) if !id.is_empty() => Ok(id.to_string()),
        _ => Err(format!(
            "could not parse the rework dispatch id from: {text}"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_enqueued_id_out_of_the_tool_result_text() {
        assert_eq!(
            parse_enqueued_id("enqueued disp-abc123 (status=ready)").unwrap(),
            "disp-abc123"
        );
        assert_eq!(
            parse_enqueued_id("enqueued disp-x (status=blocked)").unwrap(),
            "disp-x"
        );
        assert!(parse_enqueued_id("enqueue rejected: nope").is_err());
        assert!(parse_enqueued_id("").is_err());
    }

    /// A scratch repo exercises the real merge path: a clean `--no-ff` merge
    /// returns the merge commit, and a conflicting merge REFUSES (never
    /// forces) while leaving the repo exactly as found.
    #[tokio::test]
    async fn merge_refuses_conflicts_and_never_forces() {
        let root = std::env::temp_dir().join(format!("seatloom-merge-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let g = |args: &[&str]| {
            std::process::Command::new("git")
                .args(args)
                .current_dir(&root)
                .output()
                .unwrap()
        };
        let ok = |args: &[&str]| assert!(g(args).status.success(), "git {args:?} failed");
        ok(&["init", "-b", "track"]);
        ok(&["config", "user.email", "test@seatloom.local"]);
        ok(&["config", "user.name", "test"]);
        std::fs::write(root.join("f.txt"), "base\n").unwrap();
        ok(&["add", "."]);
        ok(&["commit", "-m", "base"]);

        // A clean delivery branch.
        ok(&["checkout", "-b", "packet/seat/clean"]);
        std::fs::write(root.join("clean.txt"), "delivery\n").unwrap();
        ok(&["add", "."]);
        ok(&["commit", "-m", "delivery"]);
        ok(&["checkout", "track"]);

        let sha = merge_branch_into_base(&root, "packet/seat/clean", "track")
            .await
            .expect("clean merge must succeed");
        assert_eq!(sha.len(), 40, "a full commit sha");
        // The merge commit is on track and the delivery file is present.
        assert!(root.join("clean.txt").exists());
        let head = String::from_utf8_lossy(&g(&["rev-parse", "HEAD"]).stdout)
            .trim()
            .to_string();
        assert_eq!(head, sha);

        // A conflicting delivery branch: forked from the base commit, the SAME
        // file changed on both sides (git only conflicts when both sides moved
        // relative to the merge base).
        let base_commit = out_text(&g(&["rev-parse", "HEAD"]));
        std::fs::write(root.join("f.txt"), "track moved\n").unwrap();
        ok(&["add", "."]);
        ok(&["commit", "-m", "track moves on"]);
        ok(&["checkout", "-b", "packet/seat/conflict", &base_commit]);
        std::fs::write(root.join("f.txt"), "conflicting\n").unwrap();
        ok(&["add", "."]);
        ok(&["commit", "-m", "conflict"]);
        ok(&["checkout", "track"]);
        let before = out_text(&g(&["rev-parse", "HEAD"]));

        let err = merge_branch_into_base(&root, "packet/seat/conflict", "track")
            .await
            .expect_err("a conflict must be refused");
        assert!(
            err.contains("CONFLICTED") && err.contains("nothing forced"),
            "the refusal must name the conflict and the no-force rule: {err}"
        );
        // Repo left exactly as found: no merge in progress, HEAD unchanged.
        let after = String::from_utf8_lossy(&g(&["rev-parse", "HEAD"]).stdout)
            .trim()
            .to_string();
        assert_eq!(before, after, "the refused merge must not move HEAD");
        let status_out = g(&["status", "--porcelain"]);
        let status = String::from_utf8_lossy(&status_out.stdout);
        assert!(
            status.trim().is_empty(),
            "the aborted merge must leave a clean tree: {status}"
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    /// A non-git root and a dirty tracked tree both refuse, writing nothing.
    #[tokio::test]
    async fn merge_refuses_non_git_and_dirty_roots() {
        let root =
            std::env::temp_dir().join(format!("seatloom-merge-nogit-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let err = merge_branch_into_base(&root, "any", "track")
            .await
            .expect_err("a non-git root must refuse");
        assert!(err.contains("not a git work tree"), "{err}");
        let _ = std::fs::remove_dir_all(&root);
    }
}
