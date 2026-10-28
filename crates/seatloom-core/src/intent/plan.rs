//! The planner — parse, resolve-or-stop, ground, propose. **Pure.**
//!
//! [`plan`] takes the gathered [`IntentFacts`], the sentence and the acting
//! actor, and returns one of three typed outcomes. It performs no I/O, so every
//! branch is unit-testable against a fixture and no branch can quietly depend on
//! a database being reachable.
//!
//! # The three outcomes
//!
//! | Outcome | When |
//! |---|---|
//! | [`IntentPlan::Proposal`] | the intent named an owner that exists and is routable |
//! | [`IntentPlan::Clarification`] | a §3.3 row fired — the system stopped and asked |
//! | [`IntentPlan::Retrieval`] | a why/evidence/recover verb — the exact reads to make, never an answer |
//!
//! # The rule this module exists to enforce
//!
//! > When the intent is unclear, the system asks. It does not guess.
//!
//! Concretely: an owner is selected **only** when the sentence named one and
//! that name matches exactly one seat. Capability tags narrow a *candidate
//! list*; they never select. A precondition becomes an edge only when it names a
//! referent that exists; otherwise it blocks confirmation and says why.

use crate::intent::facts::IntentFacts;
use crate::intent::parser::{self, IntentParse};
use crate::intent::schema::{
    Clarification, ClarificationChoice, EvidenceLink, Intent, IntentObjectType, IntentVerb,
    Proposal, ProposedObject, RetrievalPlan, SeatRationale, SuggestedDependency,
};

/// What the planner decided.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntentPlan {
    /// A reviewable proposal. **Nothing has been written.**
    Proposal(Box<Proposal>),
    /// The flow stopped at a question.
    Clarification(Clarification),
    /// The exact retrieval plan for a why/evidence/recover verb.
    Retrieval(RetrievalPlan),
}

/// Plan one intent. Pure over `facts`.
pub fn plan(facts: &IntentFacts, input: &str, actor: &str) -> IntentPlan {
    let intent = match parser::parse(input) {
        IntentParse::Parsed(intent) => intent,
        IntentParse::UnsupportedVerb { supported, .. } => {
            return IntentPlan::Clarification(Clarification {
                rule_id: "contract-3.3-verb-unsupported".to_string(),
                question: "这句话里没有我能确定执行的动作。你要做下面哪一种?".to_string(),
                reason: "no supported verb was recognised in the intent".to_string(),
                choices: supported,
            });
        }
        IntentParse::AmbiguousVerb { candidates, .. } => {
            return IntentPlan::Clarification(Clarification {
                rule_id: "contract-3.3-verb-ambiguous".to_string(),
                question: "这句话里出现了多个动作,你要哪一个?".to_string(),
                reason: format!(
                    "verbs matched: {}",
                    candidates
                        .iter()
                        .map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                choices: candidates
                    .iter()
                    .map(|v| ClarificationChoice {
                        value: v.as_str().to_string(),
                        label: format!("{} — {}", v.as_str(), v.one_line_example()),
                        why: "the sentence contains this verb's phrase".to_string(),
                    })
                    .collect(),
            });
        }
        IntentParse::ObjectTypeUnclear {
            verb,
            choices,
            reason,
            ..
        } => {
            return IntentPlan::Clarification(Clarification {
                rule_id: "contract-3.3-object-type-unclear".to_string(),
                question: format!("要{verb}哪一种对象?"),
                reason,
                choices,
            });
        }
        IntentParse::Unparsed { reason, .. } => {
            return IntentPlan::Clarification(Clarification {
                rule_id: "contract-3.3-unparsed".to_string(),
                question: "这句话我没法解析成任何支持的动作。".to_string(),
                reason,
                choices: parser::verb_choices(),
            });
        }
    };

    if !intent.verb.needs_object() {
        return IntentPlan::Retrieval(retrieval_plan(&intent));
    }

    match select_owner(facts, &intent) {
        OwnerSelection::Resolved(seat_id) => {
            let seat = facts
                .seat(&seat_id)
                .expect("select_owner only returns ids present in facts");
            IntentPlan::Proposal(Box::new(build_proposal(facts, &intent, seat, actor)))
        }
        OwnerSelection::Clarify(clarification) => IntentPlan::Clarification(clarification),
    }
}

// =============================================================================
// Owner selection — the only place a seat is chosen
// =============================================================================

enum OwnerSelection {
    Resolved(String),
    Clarify(Clarification),
}

/// Choose the owner, or stop and ask.
///
/// The order matters and is stated: an explicitly named owner is resolved by
/// exact match first; failing that, a latin token that is *exactly* a seat name
/// or seat id; failing that, the flow stops and asks. Capability tags never
/// select — they only order a candidate list the user confirms.
fn select_owner(facts: &IntentFacts, intent: &Intent) -> OwnerSelection {
    let project = &facts.project_id;

    if let Some(hint) = intent.owner_hint.as_deref() {
        let matches = exact_seat_matches(facts, hint);
        return match matches.as_slice() {
            [] => OwnerSelection::Clarify(Clarification {
                rule_id: "contract-3.3-owner-not-found".to_string(),
                question: format!("没有叫做 \"{hint}\" 的 seat。"),
                reason: format!(
                    "the intent named owner {hint:?}, which matches no seat name and no seat id"
                ),
                choices: seat_choices(facts, "a seat that exists in this project"),
            }),
            [only] => finish_selection(facts, intent, only),
            many => OwnerSelection::Clarify(Clarification {
                rule_id: "contract-3.3-owner-ambiguous".to_string(),
                question: format!("\"{hint}\" 匹配到多个 seat,你要哪一个?"),
                reason: format!(
                    "the owner name matched {} seats: {}",
                    many.len(),
                    many.iter()
                        .map(|s| s.seat_id.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                choices: many
                    .iter()
                    .map(|s| ClarificationChoice {
                        value: s.seat_id.clone(),
                        label: format!("{} ({})", s.name, s.seat_id),
                        why: format!(
                            "matched the named owner {:?} exactly (seats.name / seats.id)",
                            hint
                        ),
                    })
                    .collect(),
            }),
        };
    }

    // No marker introduced an owner. An exact seat-name/seat-id token in the
    // sentence is still an explicit statement by the operator, so it resolves.
    let token_matches: Vec<&crate::intent::facts::SeatFact> = intent
        .terms
        .iter()
        .filter_map(|term| exact_seat_matches(facts, term).into_iter().next())
        .collect();
    let mut unique: Vec<&crate::intent::facts::SeatFact> = Vec::new();
    for seat in token_matches {
        if !unique.iter().any(|s| s.seat_id == seat.seat_id) {
            unique.push(seat);
        }
    }
    match unique.as_slice() {
        [] => {}
        [only] => return finish_selection(facts, intent, only),
        _ => {
            return OwnerSelection::Clarify(Clarification {
                rule_id: "contract-3.3-owner-ambiguous".to_string(),
                question: "这句话里出现了多个 seat 名字,你要派给谁?".to_string(),
                reason: format!(
                    "seat names/ids matched exactly: {}",
                    unique
                        .iter()
                        .map(|s| s.seat_id.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                choices: unique
                    .iter()
                    .map(|s| ClarificationChoice {
                        value: s.seat_id.clone(),
                        label: format!("{} ({})", s.name, s.seat_id),
                        why: "the intent contains this seat's name or id as an exact token"
                            .to_string(),
                    })
                    .collect(),
            });
        }
    }

    // The owner was omitted. Ask — with the computed candidate list and why each
    // is a candidate (§3.3 row 4). Capability tags order the list; they never
    // select.
    let routable = facts.routable_seats();
    if routable.is_empty() {
        return OwnerSelection::Clarify(Clarification {
            rule_id: "contract-3.3-no-capable-seat".to_string(),
            question: "No eligible seat found".to_string(),
            reason: blocked_constraints(facts, project),
            choices: vec![],
        });
    }

    let stated_tags = stated_capability_tags(facts, intent);
    let mut ordered = routable;
    ordered.sort_by(|a, b| {
        let a_matched = matched_count(a, &stated_tags);
        let b_matched = matched_count(b, &stated_tags);
        b_matched
            .cmp(&a_matched)
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| a.seat_id.cmp(&b.seat_id))
    });

    OwnerSelection::Clarify(Clarification {
        rule_id: "contract-3.3-owner-omitted".to_string(),
        question: "派给哪个 seat?".to_string(),
        reason: if stated_tags.is_empty() {
            "the intent named no owner and stated no capability requirement, so no seat is excluded"
                .to_string()
        } else {
            format!(
                "the intent named no owner; capability tags stated: {}",
                stated_tags.join(", ")
            )
        },
        choices: ordered
            .iter()
            .map(|s| ClarificationChoice {
                value: s.seat_id.clone(),
                label: format!("{} ({})", s.name, s.seat_id),
                why: candidate_why(s, &stated_tags, project),
            })
            .collect(),
    })
}

/// A seat was named. Resolve it, or explain why it cannot take the work.
fn finish_selection(
    facts: &IntentFacts,
    intent: &Intent,
    seat: &crate::intent::facts::SeatFact,
) -> OwnerSelection {
    if let Some(reason) = seat.not_routable_reason(&facts.project_id) {
        let _ = intent;
        return OwnerSelection::Clarify(Clarification {
            rule_id: "contract-3.3-no-capable-seat".to_string(),
            question: format!("No eligible seat found: {} ({})", seat.name, seat.seat_id),
            reason,
            choices: vec![],
        });
    }
    OwnerSelection::Resolved(seat.seat_id.clone())
}

/// Exact matches for a name or seat id. Case-insensitive for ASCII (the seat
/// registry's names are lowercase latin); never fuzzy.
fn exact_seat_matches<'a>(
    facts: &'a IntentFacts,
    hint: &str,
) -> Vec<&'a crate::intent::facts::SeatFact> {
    let needle = hint.trim().to_ascii_lowercase();
    facts
        .seats
        .iter()
        .filter(|s| {
            s.name.to_ascii_lowercase() == needle || s.seat_id.to_ascii_lowercase() == needle
        })
        .collect()
}

fn seat_choices(facts: &IntentFacts, why: &str) -> Vec<ClarificationChoice> {
    facts
        .seats
        .iter()
        .map(|s| ClarificationChoice {
            value: s.seat_id.clone(),
            label: format!("{} ({})", s.name, s.seat_id),
            why: why.to_string(),
        })
        .collect()
}

/// The capability tags the intent stated **exactly** — the intersection of the
/// sentence's latin tokens with the project's own tag vocabulary. A token that
/// is not a known tag is not a requirement; it is just a word.
fn stated_capability_tags(facts: &IntentFacts, intent: &Intent) -> Vec<String> {
    let known = facts.known_capability_tags();
    intent
        .terms
        .iter()
        .filter(|t| known.contains(*t))
        .cloned()
        .collect()
}

fn matched_count(seat: &crate::intent::facts::SeatFact, stated: &[String]) -> usize {
    stated
        .iter()
        .filter(|t| seat.capability_tags.iter().any(|tag| tag == *t))
        .count()
}

fn candidate_why(
    seat: &crate::intent::facts::SeatFact,
    stated: &[String],
    project: &str,
) -> String {
    let matched: Vec<&String> = stated
        .iter()
        .filter(|t| seat.capability_tags.iter().any(|tag| tag == *t))
        .collect();
    if stated.is_empty() {
        format!("active and bound to project {project:?}; no capability requirement was stated, so no seat is excluded")
    } else if matched.len() == stated.len() {
        format!(
            "active and bound to project {project:?}; carries every stated capability tag: {}",
            matched
                .iter()
                .map(|t| t.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )
    } else {
        let missing: Vec<&str> = stated
            .iter()
            .filter(|t| !seat.capability_tags.iter().any(|tag| tag == *t))
            .map(|t| t.as_str())
            .collect();
        let carried = if matched.is_empty() {
            "none of the stated capability tags".to_string()
        } else {
            format!(
                "{} of the stated capability tags",
                matched
                    .iter()
                    .map(|t| t.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        format!(
            "active and bound to project {project:?}; carries {carried}; missing: {}",
            missing.join(", ")
        )
    }
}

/// Every reason no seat is routable, listed (§3.3 "list blocked constraints").
fn blocked_constraints(facts: &IntentFacts, project: &str) -> String {
    if facts.seats.is_empty() {
        return format!("the seats table is empty for project {project:?}");
    }
    facts
        .seats
        .iter()
        .map(|s| {
            s.not_routable_reason(project)
                .unwrap_or_else(|| "routable".to_string())
        })
        .collect::<Vec<_>>()
        .join("; ")
}

// =============================================================================
// Retrieval plan (why / evidence / recover)
// =============================================================================

fn retrieval_plan(intent: &Intent) -> RetrievalPlan {
    let reads = match intent.verb {
        IntentVerb::Explain | IntentVerb::FindEvidence => vec![
            "get_knowledge_tree — resolve the topic id from the hint (never guess an id)"
                .to_string(),
            "recall (depth l0) over that topic with the query terms".to_string(),
            "recall (depth l1/l2) only for the rows the answer rests on".to_string(),
            "query_ledger — exact ledger/artifact matches before any interpretation".to_string(),
        ],
        IntentVerb::Recover => vec![
            "list the project's dispatches — find what is in_flight or blocked".to_string(),
            "get_dispatch on that id — its dependency edges and each referent's existence"
                .to_string(),
            "query_ledger — the last lifecycle events of that dispatch".to_string(),
        ],
        // A creating verb never reaches here.
        IntentVerb::Create | IntentVerb::Route => vec![],
    };
    RetrievalPlan {
        verb: intent.verb,
        topic_hint: intent.qualifiers.topic.clone(),
        recall_query_terms: intent.terms.clone(),
        reads,
        note: "This tool does not compose an answer. Make the reads above and build the answer \
               only from what they return. If they return nothing, the result is `Evidence \
               incomplete` plus the list of sources actually searched — never a composed answer \
               (contract §3.2)."
            .to_string(),
    }
}

// =============================================================================
// Proposal builder — every field from a real read
// =============================================================================

fn build_proposal(
    facts: &IntentFacts,
    intent: &Intent,
    seat: &crate::intent::facts::SeatFact,
    actor: &str,
) -> Proposal {
    let stated_tags = stated_capability_tags(facts, intent);
    let precondition = resolve_precondition(facts, intent.qualifiers.precondition.as_deref());
    let object_kind = intent.object_type.unwrap_or(IntentObjectType::Dispatch);
    let suggested = precondition.edge.clone().into_iter().collect::<Vec<_>>();

    let blocking = precondition
        .unresolved_reason
        .as_ref()
        .map(|reason| Clarification {
            rule_id: "contract-3.3-precondition-unresolved".to_string(),
            question: "这条前置条件我没法变成一条依赖边。".to_string(),
            reason: reason.clone(),
            choices: vec![
            ClarificationChoice {
                value: "name-the-referent".to_string(),
                label: "点名前置对象(给出 dispatch / event / artifact 的 id)".to_string(),
                why: "a dependency edge needs a referent id; the clause names none that exists"
                    .to_string(),
            },
            ClarificationChoice {
                value: "confirm-without-precondition".to_string(),
                label: "不要这条前置,直接派".to_string(),
                why: "the write path can create the dispatch without the edge; the operator must \
                      say so explicitly, because dropping a precondition silently is forbidden"
                    .to_string(),
            },
            ClarificationChoice {
                value: "cancel".to_string(),
                label: "取消".to_string(),
                why: "nothing is written until the operator confirms".to_string(),
            },
        ],
        });

    let mut actions = vec!["edit".to_string(), "cancel".to_string()];
    if blocking.is_none() {
        actions.insert(0, "confirm".to_string());
    }

    Proposal {
        intent_restated: format!(
            "{} a {} for {} ({}): {}",
            intent.verb,
            object_kind,
            seat.name,
            seat.seat_id,
            one_line(&intent.raw)
        ),
        proposed_object: ProposedObject {
            kind: object_kind,
            title: one_line(&intent.raw),
            payload_note: intent.raw.clone(),
            suggested_dependencies: suggested,
        },
        ownership: crate::intent::schema::Ownership {
            seat_id: seat.seat_id.clone(),
            seat_name: seat.name.clone(),
            role: seat.role.clone(),
            capability_match: seat
                .capability_tags
                .iter()
                .filter(|t| stated_tags.contains(t))
                .cloned()
                .collect(),
            authority_docs: seat.authority_docs.clone(),
            constraints: seat.constraints.clone(),
            active_delegation_id: seat.active_delegation_id.clone(),
        },
        evidence: evidence(facts, seat, &precondition),
        budget: budget(facts, seat, intent),
        impact: impact(seat),
        routing_rationale: rationale(facts, seat, intent),
        precondition,
        blocking_clarification: blocking,
        actions,
        actor: actor.to_string(),
    }
}

fn evidence(
    facts: &IntentFacts,
    seat: &crate::intent::facts::SeatFact,
    precondition: &crate::intent::schema::PreconditionResolution,
) -> Vec<EvidenceLink> {
    let mut out = vec![
        EvidenceLink {
            label: format!(
                "seat {} ({}) status={} tags=[{}]",
                seat.name,
                seat.seat_id,
                seat.status,
                seat.capability_tags.join(", ")
            ),
            source: "seats".to_string(),
        },
        EvidenceLink {
            label: format!(
                "role binding: role={} authority_docs=[{}] constraints=[{}]",
                seat.role.as_deref().unwrap_or("<none>"),
                seat.authority_docs.join(", "),
                seat.constraints.join(", ")
            ),
            source: "project_role_bindings".to_string(),
        },
        EvidenceLink {
            label: format!(
                "in-flight dispatches for {}: {}",
                seat.seat_id,
                facts.in_flight(seat_id_of(seat))
            ),
            source: "dispatches".to_string(),
        },
        EvidenceLink {
            label: format!(
                "project budgets: worker={:?} supervisor={:?}",
                facts.worker_budget_tokens, facts.supervisor_budget_tokens
            ),
            source: "projects".to_string(),
        },
    ];
    if let Some(edge) = &precondition.edge {
        out.push(EvidenceLink {
            label: format!(
                "precondition referent {} exists in {}",
                edge.dep_value,
                edge.dep_kind.referent_table()
            ),
            source: edge.dep_kind.referent_table().to_string(),
        });
    }
    out
}

fn budget(
    facts: &IntentFacts,
    seat: &crate::intent::facts::SeatFact,
    intent: &Intent,
) -> crate::intent::schema::BudgetFacts {
    let in_flight = facts.in_flight(seat_id_of(seat));
    crate::intent::schema::BudgetFacts {
        project_worker_budget_tokens: facts.worker_budget_tokens,
        project_supervisor_budget_tokens: facts.supervisor_budget_tokens,
        seat_in_flight_dispatches: in_flight,
        stated_hint: intent.qualifiers.budget_hint.clone(),
        note: format!(
            "worker/supervisor budgets are projects.worker_budget_tokens / \
             projects.supervisor_budget_tokens for project {:?}; the load figure is the count of \
             this seat's in_flight dispatches in dispatches. A stated hint is the operator's own \
             number, reported as stated and never substituted for these reads.",
            facts.project_id
        ),
    }
}

fn impact(seat: &crate::intent::facts::SeatFact) -> Vec<String> {
    vec![
        format!(
            "one dispatches row is inserted for {} with status 'ready', or 'blocked' when a \
             dependency referent does not exist yet",
            seat.seat_id
        ),
        "one SeatDispatchIssued event is appended to canonical_events".to_string(),
        "the target seat's waker is notified, so the seat pulls the work autonomously".to_string(),
        "nothing else changes: no merge, no deploy, no service restart".to_string(),
    ]
}

fn rationale(
    facts: &IntentFacts,
    selected: &crate::intent::facts::SeatFact,
    intent: &Intent,
) -> Vec<SeatRationale> {
    let stated_tags = stated_capability_tags(facts, intent);
    let named = intent
        .owner_hint
        .clone()
        .or_else(|| {
            intent
                .terms
                .iter()
                .find(|t| exact_seat_matches(facts, t).len() == 1)
                .cloned()
        })
        .unwrap_or_default();

    facts
        .seats
        .iter()
        .map(|s| {
            let is_selected = s.seat_id == selected.seat_id;
            let matched: Vec<String> = s
                .capability_tags
                .iter()
                .filter(|t| stated_tags.contains(t))
                .cloned()
                .collect();
            let excluding = if is_selected {
                None
            } else if let Some(reason) = s.not_routable_reason(&facts.project_id) {
                Some(reason)
            } else if !named.is_empty() {
                Some(format!("the intent named owner {named:?}"))
            } else {
                Some("not the named owner".to_string())
            };
            SeatRationale {
                seat_id: s.seat_id.clone(),
                seat_name: s.name.clone(),
                eligible: s.is_routable(),
                matched_capabilities: matched,
                excluding_constraint: excluding,
                role: s.role.clone(),
                authority_docs: s.authority_docs.clone(),
                constraints: s.constraints.clone(),
                in_flight_dispatches: facts.in_flight(&s.seat_id),
                selected: is_selected,
                source: "seats + project_role_bindings + dispatches".to_string(),
            }
        })
        .collect()
}

// =============================================================================
// Precondition resolution — an edge or a stop, never a drop
// =============================================================================

fn resolve_precondition(
    facts: &IntentFacts,
    clause: Option<&str>,
) -> crate::intent::schema::PreconditionResolution {
    let Some(clause) = clause else {
        return crate::intent::schema::PreconditionResolution {
            raw: None,
            edge: None,
            matched_rule: None,
            unresolved_reason: None,
        };
    };
    let trimmed = clause.trim();

    // Rule 1 — the clause is exactly a dispatch id.
    if facts.dispatch_ids.contains(trimmed) {
        return resolved(
            clause,
            SuggestedDependency {
                dep_kind: crate::intent::schema::DepKind::Dispatch,
                dep_value: trimmed.to_string(),
            },
            "the clause is exactly a dispatch id (dispatches.id)",
        );
    }
    // Rule 2 — the clause is exactly a dispatch title.
    if facts.dispatch_titles.contains(trimmed) {
        return resolved(
            clause,
            SuggestedDependency {
                dep_kind: crate::intent::schema::DepKind::Dispatch,
                dep_value: trimmed.to_string(),
            },
            "the clause is exactly a dispatch title (dispatches.title)",
        );
    }
    // Rule 3 — the clause names a referent id that exists.
    for (kind, value) in crate::intent::facts::precondition_candidates(trimmed) {
        if facts.referent_exists(kind, &value) {
            return resolved(
                clause,
                SuggestedDependency {
                    dep_kind: kind,
                    dep_value: value.clone(),
                },
                &format!(
                    "the clause names {} {}, which exists ({})",
                    kind,
                    value,
                    kind.referent_table()
                ),
            );
        }
    }

    crate::intent::schema::PreconditionResolution {
        raw: Some(clause.to_string()),
        edge: None,
        matched_rule: None,
        unresolved_reason: Some(format!(
            "a dependency edge needs a referent id, and the clause {clause:?} names no existing \
             dispatch id, dispatch title, event id or artifact id. Name the referent explicitly \
             (for example '等 ev-... 落地'), or confirm without the precondition — dropping it \
             silently is not an option."
        )),
    }
}

fn resolved(
    clause: &str,
    edge: SuggestedDependency,
    rule: &str,
) -> crate::intent::schema::PreconditionResolution {
    crate::intent::schema::PreconditionResolution {
        raw: Some(clause.to_string()),
        edge: Some(edge),
        matched_rule: Some(rule.to_string()),
        unresolved_reason: None,
    }
}

// =============================================================================
// Small helpers
// =============================================================================

fn seat_id_of(seat: &crate::intent::facts::SeatFact) -> &str {
    &seat.seat_id
}

/// One line, bounded. Used for the proposal's title so a pasted paragraph cannot
/// become a routing summary.
fn one_line(s: &str) -> String {
    let collapsed = s.split_whitespace().collect::<Vec<_>>().join(" ");
    let char_count = collapsed.chars().count();
    if char_count <= 80 {
        collapsed
    } else {
        let head: String = collapsed.chars().take(77).collect();
        format!("{head}...")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intent::tests_support::facts;

    #[test]
    fn a_named_routable_owner_produces_a_proposal_and_writes_nothing() {
        let out = plan(
            &facts(),
            "给 nimbus 派个活,把唤醒服务兜底",
            "human:supervisor",
        );
        let IntentPlan::Proposal(p) = out else {
            panic!("expected a proposal, got {out:?}");
        };
        assert_eq!(p.ownership.seat_id, "seat-nimbus-001");
        assert_eq!(p.actor, "human:supervisor");
        assert_eq!(p.actions, vec!["confirm", "edit", "cancel"]);
        // Every budget number is a real column value, never a constant.
        assert_eq!(p.budget.project_worker_budget_tokens, Some(8192));
        assert_eq!(p.budget.seat_in_flight_dispatches, 2);
        // The rationale names the excluded seats with a fact, not an adjective.
        let flux = p
            .routing_rationale
            .iter()
            .find(|r| r.seat_id == "seat-flux-001")
            .unwrap();
        assert!(!flux.selected);
        assert!(flux
            .excluding_constraint
            .as_deref()
            .unwrap()
            .contains("nimbus"));
        let onyx = p
            .routing_rationale
            .iter()
            .find(|r| r.seat_id == "seat-onyx-001")
            .unwrap();
        assert!(!onyx.eligible);
        assert!(onyx
            .excluding_constraint
            .as_deref()
            .unwrap()
            .contains("status"));
    }

    #[test]
    fn an_owner_that_does_not_exist_asks_and_lists_the_real_seats() {
        let out = plan(&facts(), "给 atlas 派个活", "human:supervisor");
        let IntentPlan::Clarification(c) = out else {
            panic!("expected a clarification, got {out:?}");
        };
        assert_eq!(c.rule_id, "contract-3.3-owner-not-found");
        assert_eq!(c.choices.len(), 3, "the real seat list, not a default");
    }

    #[test]
    fn an_omitted_owner_asks_with_the_candidate_list_ordered_by_stated_tags() {
        let out = plan(&facts(), "派个活做 rust 的活", "human:supervisor");
        let IntentPlan::Clarification(c) = out else {
            panic!("expected a clarification, got {out:?}");
        };
        assert_eq!(c.rule_id, "contract-3.3-owner-omitted");
        assert_eq!(
            c.choices[0].value, "seat-nimbus-001",
            "carries the stated tag"
        );
        assert!(c.choices[0].why.contains("rust"));
    }

    #[test]
    fn a_retired_seat_named_explicitly_is_refused_with_the_fact() {
        let out = plan(&facts(), "把一个工作项派给 onyx", "human:supervisor");
        let IntentPlan::Clarification(c) = out else {
            panic!("expected a clarification, got {out:?}");
        };
        assert_eq!(c.rule_id, "contract-3.3-no-capable-seat");
        assert!(c.reason.contains("seats.status"));
        assert!(c.choices.is_empty(), "no action is offered");
    }

    #[test]
    fn a_precondition_that_names_a_real_event_becomes_an_edge() {
        let out = plan(
            &facts(),
            "给 nimbus 派个活,等 ev-x 落地再动",
            "human:supervisor",
        );
        let IntentPlan::Proposal(p) = out else {
            panic!("expected a proposal, got {out:?}");
        };
        let edge = p.precondition.edge.expect("the edge is attached");
        assert_eq!(edge.dep_value, "ev-x");
        assert_eq!(p.proposed_object.suggested_dependencies.len(), 1);
        assert_eq!(p.actions[0], "confirm");
        assert!(p.blocking_clarification.is_none());
    }

    #[test]
    fn a_precondition_that_names_nothing_blocks_confirmation() {
        let out = plan(
            &facts(),
            "给 nimbus 派个活,等 Flux 验证通过再动",
            "human:supervisor",
        );
        let IntentPlan::Proposal(p) = out else {
            panic!("expected a proposal, got {out:?}");
        };
        assert!(p.precondition.edge.is_none());
        assert!(p.precondition.unresolved_reason.is_some());
        let blocking = p.blocking_clarification.expect("confirmation is blocked");
        assert_eq!(blocking.rule_id, "contract-3.3-precondition-unresolved");
        assert!(!p.actions.contains(&"confirm".to_string()));
    }

    #[test]
    fn a_precondition_whose_clause_is_exactly_a_dispatch_title_becomes_an_edge() {
        // Exact match only: "Verify the gate 落地" must NOT match the title
        // "Verify the gate", because a near-match is a guess.
        let out = plan(
            &facts(),
            "给 flux 派个活,等 Verify the gate 再动",
            "human:supervisor",
        );
        let IntentPlan::Proposal(p) = out else {
            panic!("expected a proposal, got {out:?}");
        };
        let edge = p.precondition.edge.expect("the title resolved");
        assert_eq!(edge.dep_kind, crate::intent::schema::DepKind::Dispatch);
        assert_eq!(edge.dep_value, "Verify the gate");
        assert!(p
            .precondition
            .matched_rule
            .as_deref()
            .unwrap()
            .contains("title"));

        let near = plan(
            &facts(),
            "给 flux 派个活,等 Verify the gate 落地再动",
            "human:supervisor",
        );
        let IntentPlan::Proposal(near) = near else {
            panic!("expected a proposal");
        };
        assert!(
            near.precondition.edge.is_none(),
            "a near-match is not a match"
        );
        assert!(near.blocking_clarification.is_some());
    }

    #[test]
    fn a_why_question_returns_a_retrieval_plan_and_never_an_answer() {
        let out = plan(
            &facts(),
            "我们为什么放弃了 passport.js?",
            "human:supervisor",
        );
        let IntentPlan::Retrieval(r) = out else {
            panic!("expected a retrieval plan, got {out:?}");
        };
        assert_eq!(r.verb, IntentVerb::Explain);
        assert!(r.reads.iter().any(|x| x.contains("recall")));
        assert!(r.note.contains("does not compose an answer"));
    }

    #[test]
    fn an_owner_matching_two_seats_asks_instead_of_picking() {
        // A seat whose NAME is another seat's id: the hint matches both, so the
        // exact-match rule cannot decide and must say so.
        let mut f = facts();
        f.seats.push(crate::intent::facts::SeatFact {
            seat_id: "flux".to_string(),
            name: "twin".to_string(),
            capability_tags: vec![],
            status: "active".to_string(),
            role: Some("twin".to_string()),
            authority_docs: vec![],
            constraints: vec![],
            active_delegation_id: None,
        });
        let out = plan(&f, "给 flux 派个活", "human:supervisor");
        let IntentPlan::Clarification(c) = out else {
            panic!("expected a clarification, got {out:?}");
        };
        assert_eq!(c.rule_id, "contract-3.3-owner-ambiguous");
        assert_eq!(c.choices.len(), 2, "both candidates, neither picked");
    }

    #[test]
    fn the_first_owner_marker_in_the_sentence_wins() {
        // "别派给 flux" is a later marker; the primary instruction is 给 nimbus.
        let out = plan(&facts(), "给 nimbus 派个活,别派给 flux", "human:supervisor");
        let IntentPlan::Proposal(p) = out else {
            panic!("expected a proposal, got {out:?}");
        };
        assert_eq!(p.ownership.seat_id, "seat-nimbus-001");
    }

    #[test]
    fn no_field_in_the_proposal_is_a_constant_string() {
        let out = plan(
            &facts(),
            "给 nimbus 派个活,把唤醒服务兜底",
            "human:supervisor",
        );
        let IntentPlan::Proposal(p) = out else {
            panic!("expected a proposal");
        };
        // The prototype's two fabricated fields must not reappear in any form.
        let all = format!("{p:?}");
        assert!(!all.contains("150,000"), "no hardcoded budget survives");
        assert!(!all.contains("seat-2"), "no hardcoded owner survives");
        assert!(
            !all.contains("nimbus is the best"),
            "no composed rationale survives"
        );
    }
}
