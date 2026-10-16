// Governed-dispatch acceptance gate (014, D2) — the operator's decide commands.
//
// The D2 loop's user-facing half (Lyra design §4): a seat finishes work, Flux
// verifies it green, the seat calls `request_acceptance` (MCP), and the operator
// — here, the SeatLoom desktop app acting as the user's agent — accepts (merge
// into the acceptance's recorded base) or rejects (auto-enqueued rework). Seats
// NEVER merge their own work; the merge happens only on an explicit user accept,
// `--no-ff` into the acceptance's base branch, and a conflict REFUSES (never
// forces).

use std::path::Path;

use seatloom_core::db::models::{AcceptanceWithDispatchRow, VerdictFacts};
use tauri::State;

use crate::dto::{AcceptanceCard, AcceptanceLineageDto, AcceptanceResult};
use crate::state::AppState;

/// The operator identity's two channels and their precedence live in
/// [`seatloom_core::config`] — one home, unit-tested without Tauri. The env var
/// named there is the SECOND channel, not the only one: the first is a
/// single-line file at `~/.seatloom/operator`, because an app launched from the
/// Launchpad has no environment at all and a correct guard must not become a wall
/// the operator can never get past. Both channels are explicit configuration by a
/// person; neither is a default, and there is no third channel.
///
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
/// is absent, red, or about some other dispatch (未验绿不可合), or the merge
/// conflicts (git's own conflict text is returned; the merge is aborted, never
/// forced).
#[tauri::command]
pub async fn cmd_accept_delivery(
    state: State<'_, AppState>,
    acceptance_id: String,
) -> Result<AcceptanceResult, String> {
    let operator = operator_ref()?;
    let row = load_pending(&state, &acceptance_id).await?;

    // Merge criterion, first half: the Flux GREEN verdict, about THIS dispatch,
    // must exist NOW.
    //
    // THE DEFECT THIS CLOSES. This used to call `event_or_artifact_exists`, which
    // answers one question — "is the referent there" — while the comment above it
    // promised green. Measured on live, three pending cards named
    // `ev-verdict-history-sessions`: a **fail** verdict about
    // `disp-history-sessions-verify`, for three dispatches Flux had never
    // verified. Existence was being read as a verdict, and a red verdict about
    // someone else's work would have merged on the operator's click.
    //
    // So the value is read and the subject is checked, and every refusal names
    // which verdict it was, what the verdict said, and why that does not count.
    // Checked again at decision time, not only at request time, because a verdict
    // can be re-issued or a card can be re-opened between the two.
    let facts = state
        .db
        .flux_verdict_facts(
            &row.acceptance.flux_verdict_ref,
            &row.acceptance.dispatch_id,
        )
        .await
        .map_err(|e| e.to_string())?;
    verdict_gate_message(&facts, &row.acceptance.flux_verdict_ref)?;

    let merge_commit = merge_branch_into_base(
        &state.repo_root,
        &row.acceptance.branch,
        &row.acceptance.base,
    )
    .await?;

    let decided = state
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
        status: decided.status,
        merge_commit: decided.merge_commit,
        rework_dispatch_id: decided.rework_dispatch_id,
        error: None,
    })
}

/// Assemble one acceptance card's lineage (acceptance-lineage Lane B): the card,
/// the dispatch it gates with the ORIGINAL task statement, the event chain, the
/// Flux verdict and its evidence, the products, the dependency edges, and the
/// rework dispatch.
///
/// Pure read — it writes nothing and appends no event (the operator's blocking
/// feedback was a DISPLAY gap, not a data gap). It calls the SAME assembler the
/// MCP `get_acceptance_lineage` tool calls (`seatloom_core::db::lineage`), so
/// the two surfaces cannot drift; there is no second query implementation.
///
/// No operator ref is required: this reads a recorded card, it does not act.
#[tauri::command]
pub async fn cmd_get_acceptance_lineage(
    state: State<'_, AppState>,
    acceptance_id: String,
) -> Result<AcceptanceLineageDto, String> {
    let lineage = seatloom_core::db::lineage::get_acceptance_lineage(&state.db, &acceptance_id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(AcceptanceLineageDto::from(lineage))
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

    let decided = state
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

    // The Flux HOLD fix: the decision response carries the landed status AND the
    // rework dispatch id, so the UI's reject path renders the rejected badge and
    // the jump-to-rework link from this response alone (a superset of the card
    // normaliser — no Mira-side change).
    Ok(AcceptanceResult {
        accepted: false,
        status: decided.status,
        merge_commit: decided.merge_commit,
        rework_dispatch_id: decided.rework_dispatch_id,
        error: None,
    })
}

// =============================================================================
// Helpers
// =============================================================================

/// The operator acting on this decision, resolved from the configuration file
/// first and the environment second.
///
/// The resolution itself — and every rule about what counts as an identity —
/// lives in [`seatloom_core::config`], so it is unit-testable without Tauri and
/// there is exactly one place that decides. This function only renders the typed
/// refusal into the message the operator sees; it never substitutes a value.
fn operator_ref() -> Result<String, String> {
    seatloom_core::config::read_operator_ref()
        .map(|r| r.value)
        .map_err(|e| seatloom_core::config::operator_ref_refusal_message(&e))
}

/// The merge criterion's verdict half, rendered for the operator.
///
/// Three separate refusals, because they call for three different actions: a
/// missing referent means the verdict vanished; a red one means Flux said no; one
/// about another dispatch means it was never about this work at all. Collapsing
/// them into "未验绿不可合" is what left Mr Zhang with a single opaque
/// "操作失败" — his words — so each names the verdict, its value, and what is
/// wrong with it.
fn verdict_gate_message(facts: &VerdictFacts, ref_id: &str) -> Result<(), String> {
    if !facts.exists {
        return Err(format!(
            "未验绿不可合:验绿依据 {ref_id:?} 不在账本里 —— 它可能被删了,或者从来没被记录过。\
             没有验绿结论,就不能合。"
        ));
    }
    if facts.event_type.as_deref() != Some("ReviewVerdictIssued") {
        return Err(format!(
            "未验绿不可合:验绿依据 {ref_id:?} 不是一条裁决事件(它是 {:?})——\
             只有 Flux 的 ReviewVerdictIssued 能当验绿依据。",
            facts
                .event_type
                .as_deref()
                .unwrap_or("一个产出物,没有事件类型")
        ));
    }
    match facts.verdict.as_deref() {
        Some("pass") => {}
        Some(value) => {
            return Err(format!(
                "未验绿不可合:验绿结论 {ref_id:?} 的结论是 {value:?},不是通过。\
                 Flux 判了不通过就不能合 —— 让他重新验,或者把打回的理由处理掉。"
            ))
        }
        None => {
            return Err(format!(
                "未验绿不可合:验绿结论 {ref_id:?} 没有写结论值。\
                 一条没写通过与否的结论,不能算通过。"
            ))
        }
    }
    if !facts.names_dispatch {
        let about = facts
            .payload_dispatch_id
            .as_deref()
            .map(|d| format!("它说的是 {d:?}"))
            .unwrap_or_else(|| "它没有说是哪条派单的".to_string());
        return Err(format!(
            "未验绿不可合:验绿结论 {ref_id:?} 不是关于这条派单的 —— {about}。\
             别的活的结论,不能当这一条的验绿依据。"
        ));
    }
    Ok(())
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
/// root, a dirty tracked tree, a missing branch, a missing BASE, a failed base
/// checkout, or a merge conflict. On conflict the merge is ABORTED, leaving the
/// repo exactly as found, and git's own conflict text is returned.
///
/// Both refs are proven to exist BEFORE anything is checked out or merged. The
/// base check used to be missing: `request_acceptance` recorded the hardcoded
/// `'track'`, and the failure only appeared here as git's "could not check out
/// base". The request-time guard now catches it first; this is the backstop that
/// keeps a bad row from ever reaching the merge.
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
    // 3b. The BASE must exist too — symmetric with the branch check above, and
    //     the direct fix for the bootstrap card whose base was the literal
    //     'track' (no such branch; the integration branch is
    //     'track/infra-foundation'). Checking here as well means a row recorded
    //     before the request-time guard existed still refuses cleanly instead of
    //     half-checking-out and failing.
    let base_here = seatloom_core::git::ops::local_branch_exists(repo_root, base)
        .await
        .map_err(|detail| format!("could not verify base {base:?}: {detail}"))?;
    if !base_here {
        return Err(format!(
            "base {base:?} does not exist in {} — the acceptance recorded a merge target that is not a branch; \
             cancel the acceptance and re-request it with the real integration branch",
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

    /// The Flux HOLD fix, pinned at the wire: the decision response is a SUPERSET
    /// of the acceptance-card normaliser — `status` and `reworkDispatchId` are
    /// present with the camelCase names the UI reads, on BOTH paths.
    #[test]
    fn acceptance_result_serializes_the_reject_path_fields() {
        let reject = AcceptanceResult {
            accepted: false,
            status: "rejected".to_string(),
            merge_commit: None,
            rework_dispatch_id: Some("disp-rework-1".to_string()),
            error: None,
        };
        let v: serde_json::Value = serde_json::to_value(&reject).unwrap();
        assert_eq!(v["accepted"], false);
        assert_eq!(v["status"], "rejected");
        assert_eq!(v["reworkDispatchId"], "disp-rework-1");
        assert!(v.get("mergeCommit").is_some());
        assert!(v.get("error").is_some());

        let accept = AcceptanceResult {
            accepted: true,
            status: "accepted".to_string(),
            merge_commit: Some("abc123".to_string()),
            rework_dispatch_id: None,
            error: None,
        };
        let v: serde_json::Value = serde_json::to_value(&accept).unwrap();
        assert_eq!(v["status"], "accepted");
        assert_eq!(v["mergeCommit"], "abc123");
        assert!(v["reworkDispatchId"].is_null());
    }

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
