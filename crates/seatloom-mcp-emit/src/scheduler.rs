//! P0 step 6 (Nimbus half) — the ready-aware scheduler core, as pure logic.
//!
//! This module is the algorithmic heart of governed dispatch (G19): readiness
//! evaluation, the per-seat pull ordering, and cycle / stall detection. It is
//! deliberately **database- and vocabulary-independent** so it can be built and
//! unit-tested against the shared contract
//! (`VALUE-DOMAIN-2026-10-08-governed-dispatch-v1`) *before* Onyx's `012`
//! migration and the three new `EventType` values land. The DB/MCP integration
//! layer (`enqueue_dispatch` / `get_next_dispatch` / `complete_dispatch`) wraps
//! these functions and supplies the only things they cannot compute themselves:
//! the live existence of dependency referents, and the atomic row+event write.
//!
//! # Why a pure core
//!
//! Everything here is a decision over *data already fetched* — it performs no
//! I/O. That keeps the rules the contract makes binding (skip-blocked,
//! highest-priority-first, one-in-flight, cycle/stall escalation) testable
//! exhaustively without a Postgres fixture, and it keeps the DB layer a thin,
//! auditable adapter rather than a place where policy hides.
//!
//! # Contract mapping (VALUE-DOMAIN-2026-10-08-governed-dispatch-v1)
//!
//! - STATUS DOMAIN (4): [`DispatchStatus`] = ready | blocked | in_flight | done.
//! - Rule 1 (one in_flight per seat): [`select_next`] returns an existing
//!   in_flight before any ready item; the database partial unique index is the
//!   actual enforcement, this is the scheduler half.
//! - Rule 4 (blocked never head-of-line-blocks): [`select_next`] filters to
//!   `ready` and skips `blocked`.
//! - Pull ordering: `priority DESC, issued_at ASC` ([`select_next`]).
//! - Rule 5 (cycle / stall detected, never hung): [`detect_cycle`] and
//!   [`classify_stall`].
//!
//! Readiness itself (contract: "a dispatch is READY iff every dependency row's
//! referent EXISTS") is evaluated by the DB layer via existence queries, never
//! by FK — Onyx's migration has no FK on `dep_value` by design. This module
//! consumes the *result* of that evaluation as [`DispatchView::status`].
//!
//! Consumed by [`crate::dispatch`], which supplies live readiness (existence
//! queries) and the atomic row+event write.

use std::collections::{HashMap, HashSet};

/// The 4-value status domain, lowercase snake_case to match the DB CHECK
/// (`ready | blocked | in_flight | done`). Kept local to the scheduler crate:
/// it is a dispatch concern, not part of A1's `event_domain` vocabulary, so it
/// does not touch the accepted enum file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchStatus {
    Ready,
    Blocked,
    InFlight,
    Done,
}

impl DispatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            DispatchStatus::Ready => "ready",
            DispatchStatus::Blocked => "blocked",
            DispatchStatus::InFlight => "in_flight",
            DispatchStatus::Done => "done",
        }
    }

    /// Parse the DB TEXT value. Returns `None` for an unknown value rather than
    /// guessing — the DB CHECK should make that impossible, but a silent
    /// fallback is exactly the defect the hand-written boundary parsers avoid.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "ready" => Some(DispatchStatus::Ready),
            "blocked" => Some(DispatchStatus::Blocked),
            "in_flight" => Some(DispatchStatus::InFlight),
            "done" => Some(DispatchStatus::Done),
            _ => None,
        }
    }
}

/// The kind of a dependency referent (`dispatch_dependencies.dep_kind`).
/// Polymorphic across three object tables — which is exactly why the schema has
/// no FK on `dep_value` and readiness is an existence query.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DepKind {
    Event,
    Artifact,
    Dispatch,
}

impl DepKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            DepKind::Event => "event",
            DepKind::Artifact => "artifact",
            DepKind::Dispatch => "dispatch",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "event" => Some(DepKind::Event),
            "artifact" => Some(DepKind::Artifact),
            "dispatch" => Some(DepKind::Dispatch),
            _ => None,
        }
    }
}

/// One dependency edge: this dispatch needs `(dep_kind, dep_value)` to exist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dependency {
    pub dep_kind: DepKind,
    pub dep_value: String,
}

/// The scheduler's view of one dispatch row, already joined to its
/// dependencies and with `status` reflecting live readiness (the DB layer
/// computes `ready` vs `blocked` by existence query before constructing this).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispatchView {
    pub id: String,
    pub target_seat_id: String,
    pub status: DispatchStatus,
    pub priority: i32,
    /// RFC-3339-sortable timestamp string; used only for the tie-break so the
    /// pure core stays free of a time dependency. The DB layer passes the
    /// `issued_at` column rendered as a sortable string.
    pub issued_at: String,
    pub dependencies: Vec<Dependency>,
}

/// The result of a pull decision for one seat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PullDecision {
    /// The seat already holds an in_flight dispatch; hand back the same one
    /// (contract rule 1 — one in_flight per seat is a fact, not a preference).
    AlreadyInFlight { dispatch_id: String },
    /// A ready dispatch was selected to become in_flight.
    Deliver { dispatch_id: String },
    /// Nothing ready and nothing in_flight. `stall` distinguishes "idle, all
    /// done / empty inbox" (not a stall) from "idle because everything is
    /// blocked" (a stall to escalate).
    Nothing { stall: bool },
}

/// Decide what, if anything, to hand the calling seat. Pure over the seat's own
/// dispatches (the DB layer passes only `seat_id`'s rows).
///
/// Order of decision, per the contract:
///   1. an existing in_flight wins (rule 1);
///   2. else the highest-priority ready item, ties broken by earliest issued_at
///      (rule 4 skips blocked implicitly by filtering to ready);
///   3. else nothing — and we classify whether that "nothing" is a stall.
pub fn select_next(seat_id: &str, dispatches: &[DispatchView]) -> PullDecision {
    let mine: Vec<&DispatchView> = dispatches
        .iter()
        .filter(|d| d.target_seat_id == seat_id)
        .collect();

    if let Some(d) = mine.iter().find(|d| d.status == DispatchStatus::InFlight) {
        return PullDecision::AlreadyInFlight {
            dispatch_id: d.id.clone(),
        };
    }

    let mut ready: Vec<&&DispatchView> = mine
        .iter()
        .filter(|d| d.status == DispatchStatus::Ready)
        .collect();
    // priority DESC, then issued_at ASC. A stable sort over a total order; the
    // id is the final tiebreak so the choice is deterministic even for equal
    // priority and timestamp.
    ready.sort_by(|a, b| {
        b.priority
            .cmp(&a.priority)
            .then_with(|| a.issued_at.cmp(&b.issued_at))
            .then_with(|| a.id.cmp(&b.id))
    });

    if let Some(best) = ready.first() {
        return PullDecision::Deliver {
            dispatch_id: best.id.clone(),
        };
    }

    // Nothing ready, nothing in_flight. It is a STALL iff the seat has at least
    // one blocked dispatch and no ready one (contract §1.3 / rule 5): the seat
    // is idle *because* work is blocked, not because its inbox is empty or done.
    let has_blocked = mine.iter().any(|d| d.status == DispatchStatus::Blocked);
    PullDecision::Nothing { stall: has_blocked }
}

/// Stall classification for one seat, independent of a pull. Used by the
/// escalation path so a stall can be reported even outside a `get_next_dispatch`
/// call. A seat stalls iff it has ≥1 blocked dispatch and 0 ready and 0
/// in_flight — idle with work it cannot reach.
pub fn classify_stall(seat_id: &str, dispatches: &[DispatchView]) -> bool {
    let mine: Vec<&DispatchView> = dispatches
        .iter()
        .filter(|d| d.target_seat_id == seat_id)
        .collect();
    let has_blocked = mine.iter().any(|d| d.status == DispatchStatus::Blocked);
    let has_ready = mine.iter().any(|d| d.status == DispatchStatus::Ready);
    let has_in_flight = mine.iter().any(|d| d.status == DispatchStatus::InFlight);
    has_blocked && !has_ready && !has_in_flight
}

/// A detected dependency cycle, reported by dispatch id in discovery order so
/// the escalation message can name the offending ring.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cycle {
    pub ids: Vec<String>,
}

/// Detect a cycle in the dispatch dependency graph, following only
/// `dep_kind == dispatch` edges (contract §1.3: "including cross-seat edges via
/// dep_kind='dispatch'"). Event and artifact dependencies cannot form a cycle
/// among dispatches, so they are not graph edges here.
///
/// # Algorithm and cost
///
/// A standard iterative DFS with a three-colour (white / grey / black) marking,
/// over the sub-graph of `dispatch`→`dispatch` edges. A grey node reached again
/// is a back edge = a cycle; the grey stack from that node back to the re-entry
/// is the ring. O(V + E) over dispatches and their dispatch-kind dependencies.
/// Run per pull it is a full walk of one project's dispatch graph; the first
/// slice accepts that (the graphs are small — a handful of in-flight hops), and
/// §1.3 explicitly allows a full walk if stated. An incremental check is a
/// later refinement, not needed to make the loop safe.
///
/// Returns the first cycle found, or `None`. Only edges whose target dispatch id
/// is present in `dispatches` are followed; a `dispatch` dependency on an id the
/// scheduler cannot see is a *blocked/unsatisfied* dependency (handled by
/// readiness), not a cycle.
pub fn detect_cycle(dispatches: &[DispatchView]) -> Option<Cycle> {
    // Adjacency: dispatch id -> ids it depends on (dispatch-kind edges only),
    // restricted to ids we actually have a node for.
    let present: HashSet<&str> = dispatches.iter().map(|d| d.id.as_str()).collect();
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for d in dispatches {
        let edges: Vec<&str> = d
            .dependencies
            .iter()
            .filter(|dep| dep.dep_kind == DepKind::Dispatch)
            .map(|dep| dep.dep_value.as_str())
            .filter(|target| present.contains(target))
            .collect();
        adj.insert(d.id.as_str(), edges);
    }

    #[derive(Clone, Copy, PartialEq)]
    enum Color {
        White,
        Grey,
        Black,
    }
    let mut color: HashMap<&str, Color> = present.iter().map(|id| (*id, Color::White)).collect();

    // Iterative DFS carrying an explicit path stack so we can slice out the ring.
    for &start in &present {
        if color[start] != Color::White {
            continue;
        }
        // stack of (node, index-into-its-adjacency-not-yet-visited)
        let mut stack: Vec<(&str, usize)> = vec![(start, 0)];
        let mut path: Vec<&str> = vec![start];
        *color.get_mut(start).unwrap() = Color::Grey;

        while let Some(&mut (node, ref mut idx)) = stack.last_mut() {
            let neighbors = &adj[node];
            if *idx < neighbors.len() {
                let next = neighbors[*idx];
                *idx += 1;
                match color[next] {
                    Color::White => {
                        *color.get_mut(next).unwrap() = Color::Grey;
                        stack.push((next, 0));
                        path.push(next);
                    }
                    Color::Grey => {
                        // Back edge: `next` is on the current path. The ring is
                        // path[pos..] where path[pos] == next.
                        let pos = path.iter().position(|p| *p == next).unwrap();
                        let ids = path[pos..].iter().map(|s| s.to_string()).collect();
                        return Some(Cycle { ids });
                    }
                    Color::Black => {}
                }
            } else {
                *color.get_mut(node).unwrap() = Color::Black;
                stack.pop();
                path.pop();
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(id: &str, seat: &str, status: DispatchStatus, pri: i32, issued: &str) -> DispatchView {
        DispatchView {
            id: id.to_string(),
            target_seat_id: seat.to_string(),
            status,
            priority: pri,
            issued_at: issued.to_string(),
            dependencies: vec![],
        }
    }

    fn dep(id: &str, seat: &str, status: DispatchStatus, deps: Vec<&str>) -> DispatchView {
        DispatchView {
            id: id.to_string(),
            target_seat_id: seat.to_string(),
            status,
            priority: 0,
            issued_at: "2026-10-08T00:00:00Z".to_string(),
            dependencies: deps
                .into_iter()
                .map(|v| Dependency {
                    dep_kind: DepKind::Dispatch,
                    dep_value: v.to_string(),
                })
                .collect(),
        }
    }

    // ---- status / dep_kind round trips -------------------------------------

    #[test]
    fn status_and_dep_kind_round_trip_and_reject_unknown() {
        for s in [
            DispatchStatus::Ready,
            DispatchStatus::Blocked,
            DispatchStatus::InFlight,
            DispatchStatus::Done,
        ] {
            assert_eq!(DispatchStatus::parse(s.as_str()), Some(s));
        }
        assert_eq!(DispatchStatus::parse("running"), None);
        for k in [DepKind::Event, DepKind::Artifact, DepKind::Dispatch] {
            assert_eq!(DepKind::parse(k.as_str()), Some(k));
        }
        assert_eq!(DepKind::parse("session"), None);
    }

    // ---- select_next: the pull algorithm -----------------------------------

    /// Contract rule 4: a blocked item is skipped, the ready one is returned —
    /// and we assert the blocked one was NOT chosen, not merely that a ready one
    /// was (packet §2.2).
    #[test]
    fn pull_skips_blocked_and_returns_ready() {
        let items = vec![
            d("disp-blocked", "seat-a", DispatchStatus::Blocked, 100, "t1"),
            d("disp-ready", "seat-a", DispatchStatus::Ready, 10, "t2"),
        ];
        match select_next("seat-a", &items) {
            PullDecision::Deliver { dispatch_id } => assert_eq!(dispatch_id, "disp-ready"),
            other => panic!("expected Deliver(disp-ready), got {other:?}"),
        }
    }

    /// Highest priority wins; ties broken by earliest issued_at.
    #[test]
    fn pull_orders_by_priority_then_issued_at() {
        let items = vec![
            d("low", "s", DispatchStatus::Ready, 1, "t5"),
            d("high-late", "s", DispatchStatus::Ready, 9, "t9"),
            d("high-early", "s", DispatchStatus::Ready, 9, "t1"),
        ];
        match select_next("s", &items) {
            PullDecision::Deliver { dispatch_id } => assert_eq!(dispatch_id, "high-early"),
            other => panic!("expected high-early, got {other:?}"),
        }
    }

    /// Contract rule 1: an existing in_flight is returned before any ready item,
    /// so a seat never holds two.
    #[test]
    fn pull_returns_existing_in_flight_before_any_ready() {
        let items = vec![
            d("already", "s", DispatchStatus::InFlight, 1, "t1"),
            d("tempting", "s", DispatchStatus::Ready, 100, "t2"),
        ];
        match select_next("s", &items) {
            PullDecision::AlreadyInFlight { dispatch_id } => assert_eq!(dispatch_id, "already"),
            other => panic!("expected AlreadyInFlight(already), got {other:?}"),
        }
    }

    /// One-in-flight is per seat: seat-b's pull ignores seat-a's rows entirely.
    #[test]
    fn pull_is_scoped_to_the_calling_seat() {
        let items = vec![
            d("a-inflight", "seat-a", DispatchStatus::InFlight, 1, "t1"),
            d("b-ready", "seat-b", DispatchStatus::Ready, 1, "t2"),
        ];
        match select_next("seat-b", &items) {
            PullDecision::Deliver { dispatch_id } => assert_eq!(dispatch_id, "b-ready"),
            other => panic!("expected b-ready, got {other:?}"),
        }
    }

    /// Nothing ready, nothing in_flight, nothing blocked -> idle, NOT a stall.
    #[test]
    fn empty_or_done_inbox_is_not_a_stall() {
        let done = vec![d("x", "s", DispatchStatus::Done, 1, "t1")];
        assert_eq!(
            select_next("s", &done),
            PullDecision::Nothing { stall: false }
        );
        assert_eq!(
            select_next("s", &[]),
            PullDecision::Nothing { stall: false }
        );
    }

    /// Nothing ready, nothing in_flight, but a blocked item exists -> STALL.
    #[test]
    fn all_blocked_nothing_ready_is_a_stall() {
        let items = vec![
            d("b1", "s", DispatchStatus::Blocked, 1, "t1"),
            d("b2", "s", DispatchStatus::Blocked, 1, "t2"),
        ];
        assert_eq!(
            select_next("s", &items),
            PullDecision::Nothing { stall: true }
        );
        assert!(classify_stall("s", &items));
    }

    /// "blocked, and something else is ready" is normal, not a stall (§1.3).
    #[test]
    fn blocked_with_something_ready_is_not_a_stall() {
        let items = vec![
            d("b", "s", DispatchStatus::Blocked, 1, "t1"),
            d("r", "s", DispatchStatus::Ready, 1, "t2"),
        ];
        assert!(!classify_stall("s", &items));
        assert!(matches!(
            select_next("s", &items),
            PullDecision::Deliver { .. }
        ));
    }

    // ---- detect_cycle ------------------------------------------------------

    #[test]
    fn no_cycle_in_a_simple_chain() {
        // c -> b -> a (a depends on nothing). No cycle.
        let items = vec![
            dep("a", "s", DispatchStatus::Ready, vec![]),
            dep("b", "s", DispatchStatus::Blocked, vec!["a"]),
            dep("c", "s", DispatchStatus::Blocked, vec!["b"]),
        ];
        assert_eq!(detect_cycle(&items), None);
    }

    /// Contract §1.3: a cross-seat cycle (A on seat-1 depends on B's dispatch,
    /// B on seat-2 depends on A's dispatch) is detected.
    #[test]
    fn cross_seat_two_cycle_is_detected() {
        let items = vec![
            dep("A", "seat-1", DispatchStatus::Blocked, vec!["B"]),
            dep("B", "seat-2", DispatchStatus::Blocked, vec!["A"]),
        ];
        let c = detect_cycle(&items).expect("cycle must be detected");
        let set: HashSet<&str> = c.ids.iter().map(|s| s.as_str()).collect();
        assert!(
            set.contains("A") && set.contains("B"),
            "ring names both: {c:?}"
        );
    }

    #[test]
    fn three_node_cycle_is_detected() {
        let items = vec![
            dep("A", "s", DispatchStatus::Blocked, vec!["B"]),
            dep("B", "s", DispatchStatus::Blocked, vec!["C"]),
            dep("C", "s", DispatchStatus::Blocked, vec!["A"]),
        ];
        assert!(detect_cycle(&items).is_some());
    }

    /// A dispatch dependency on an id the scheduler cannot see is unsatisfied
    /// (blocked), NOT a cycle — only edges to present nodes are graph edges.
    #[test]
    fn dependency_on_absent_dispatch_is_not_a_cycle() {
        let items = vec![dep("A", "s", DispatchStatus::Blocked, vec!["ghost"])];
        assert_eq!(detect_cycle(&items), None);
    }

    /// Event/artifact dependencies are not dispatch-graph edges, so they can
    /// never fabricate a cycle.
    #[test]
    fn non_dispatch_dependencies_are_not_graph_edges() {
        let mut a = dep("A", "s", DispatchStatus::Blocked, vec![]);
        a.dependencies.push(Dependency {
            dep_kind: DepKind::Event,
            dep_value: "A".to_string(), // self-id as an EVENT dep: still no cycle
        });
        assert_eq!(detect_cycle(&[a]), None);
    }
}
