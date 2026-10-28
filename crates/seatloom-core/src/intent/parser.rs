//! The deterministic parser — **no LLM in this step** (contract §3.2).
//!
//! The parser is a pure function of the sentence. It performs no I/O, holds no
//! state, and resolves nothing against the database: it lifts verbs, object
//! types, an owner *hint* and verbatim qualifiers out of the text, and it
//! reports what it could not lift as a typed result rather than repairing the
//! gap by inference.
//!
//! # Why phrase tables and not a model
//!
//! The contract's load-bearing rule is "when the intent is unclear, the system
//! asks; it does not guess" (§3.3). A model would produce a plausible parse for
//! every input, which is precisely the failure mode. A phrase table fails
//! *loudly and identically* on every run, and its vocabulary is stated in one
//! place below so a reviewer can read every decision it can make.
//!
//! # Matching rules
//!
//! 1. **Maximal munch.** Overlapping phrases resolve to the longest non-
//!    overlapping match (`任务单` beats `任务`; `assign to` beats `assign`).
//!    Without this, one sentence could match two object types and the parser
//!    would have to pick — which is guessing.
//! 2. **Word boundaries for latin phrases.** `add` must not match inside
//!    `address`. CJK phrases have no word boundaries and are matched as
//!    substrings.
//! 3. **Exact, never fuzzy.** The owner hint is taken from an explicit marker
//!    (`派给`, `给 … 派`, `assign to`) or, failing that, left to the resolver,
//!    which matches it against the seat registry by exact name or seat id.
//! 4. **Verbatim qualifiers.** A precondition clause is captured as typed. It
//!    becomes a dependency edge only if it names a real referent.

use std::collections::BTreeSet;

use crate::intent::schema::{
    ClarificationChoice, Intent, IntentObjectType, IntentQualifiers, IntentVerb,
};

// =============================================================================
// Public API
// =============================================================================

/// The result of parsing one sentence. Every variant is a typed result; none is
/// an error string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntentParse {
    /// Parsed far enough to attempt resolution.
    Parsed(Intent),
    /// No supported verb was recognised (§3.3 row 1).
    UnsupportedVerb {
        input: String,
        supported: Vec<ClarificationChoice>,
    },
    /// Two or more supported verbs matched (§3.3, stated addition — ask, never
    /// pick the first).
    AmbiguousVerb {
        input: String,
        candidates: Vec<IntentVerb>,
    },
    /// The object type is missing or ambiguous (§3.3 row 5).
    ObjectTypeUnclear {
        input: String,
        verb: IntentVerb,
        choices: Vec<ClarificationChoice>,
        reason: String,
    },
    /// Nothing at all could be lifted out of the sentence.
    Unparsed { input: String, reason: String },
}

/// Parse one sentence. Pure: no I/O, no clock, no randomness.
pub fn parse(input: &str) -> IntentParse {
    let raw = input.trim();
    if raw.is_empty() {
        return IntentParse::Unparsed {
            input: raw.to_string(),
            reason: "the intent is empty".to_string(),
        };
    }

    let has_route_marker = any_phrase(raw, ROUTE_PHRASES);
    let verbs = distinct(matched_verbs(raw, has_route_marker));
    let verb = match verbs.as_slice() {
        [] => {
            return IntentParse::UnsupportedVerb {
                input: raw.to_string(),
                supported: verb_choices(),
            };
        }
        [only] => *only,
        many => {
            return IntentParse::AmbiguousVerb {
                input: raw.to_string(),
                candidates: many.to_vec(),
            };
        }
    };

    let objects = distinct(matched_objects(raw));
    let object_type = if verb.needs_object() {
        match objects.as_slice() {
            [] => {
                return IntentParse::ObjectTypeUnclear {
                    input: raw.to_string(),
                    verb,
                    choices: object_choices(),
                    reason: "no object type was stated".to_string(),
                };
            }
            [only] => Some(*only),
            many => {
                return IntentParse::ObjectTypeUnclear {
                    input: raw.to_string(),
                    verb,
                    choices: object_choices(),
                    reason: format!(
                        "more than one object type matched: {}",
                        many.iter()
                            .map(|o| o.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                };
            }
        }
    } else {
        // A why/evidence/recover verb does not create an object, so the object
        // type does not gate the flow. It is recorded when exactly one phrase
        // matched (a fact) and left unset otherwise — never inferred.
        match objects.as_slice() {
            [only] => Some(*only),
            _ => None,
        }
    };

    IntentParse::Parsed(Intent {
        verb,
        object_type,
        owner_hint: owner_hint(raw),
        qualifiers: qualifiers(raw),
        terms: latin_terms(raw),
        raw: raw.to_string(),
    })
}

// =============================================================================
// Vocabulary — every decision the parser can make is stated here
// =============================================================================

/// Verb surface forms, longest first. Both Chinese and English, because the
/// operator types Chinese and the seats' own vocabulary is English.
///
/// **The shared-phrase rule.** `派单`, `派活` and `派个活` appear in both this
/// table and [`OBJECT_PHRASES`], because they name an action *and* the object it
/// produces. When the sentence also carries a route marker ("把一个派单派给
/// flux", "派给 onyx 一个派单") the shared phrase is read as the **object**
/// only, and the route marker is the verb — otherwise every routing sentence
/// that names a dispatch would read as "create". The rule is stated here, in one
/// place, so a reviewer can read every decision the parser can make; it is not
/// an inference about the world.
const VERB_PHRASES: &[(&str, IntentVerb)] = &[
    // create
    ("创建一个", IntentVerb::Create),
    ("新建一个", IntentVerb::Create),
    ("派个活", IntentVerb::Create),
    ("派活", IntentVerb::Create),
    ("派单", IntentVerb::Create),
    ("加一个", IntentVerb::Create),
    ("建一个", IntentVerb::Create),
    ("创建", IntentVerb::Create),
    ("新建", IntentVerb::Create),
    ("添加", IntentVerb::Create),
    ("create", IntentVerb::Create),
    ("add", IntentVerb::Create),
    // route
    ("转派给", IntentVerb::Route),
    ("分配给", IntentVerb::Route),
    ("移交给", IntentVerb::Route),
    ("assign to", IntentVerb::Route),
    ("route to", IntentVerb::Route),
    ("delegate to", IntentVerb::Route),
    ("派给", IntentVerb::Route),
    ("assign", IntentVerb::Route),
    ("route", IntentVerb::Route),
    // explain
    ("解释一下", IntentVerb::Explain),
    ("说明一下", IntentVerb::Explain),
    ("为什么", IntentVerb::Explain),
    ("为何", IntentVerb::Explain),
    ("解释", IntentVerb::Explain),
    ("说明", IntentVerb::Explain),
    ("explain", IntentVerb::Explain),
    ("why did", IntentVerb::Explain),
    ("why", IntentVerb::Explain),
    // find-evidence
    ("找一下证据", IntentVerb::FindEvidence),
    ("找出处", IntentVerb::FindEvidence),
    ("找证据", IntentVerb::FindEvidence),
    ("证据在哪", IntentVerb::FindEvidence),
    ("证据", IntentVerb::FindEvidence),
    ("find evidence", IntentVerb::FindEvidence),
    ("evidence for", IntentVerb::FindEvidence),
    ("evidence", IntentVerb::FindEvidence),
    // recover
    ("接着做", IntentVerb::Recover),
    ("恢复", IntentVerb::Recover),
    ("接着", IntentVerb::Recover),
    ("继续", IntentVerb::Recover),
    ("recover", IntentVerb::Recover),
    ("resume", IntentVerb::Recover),
    ("continue", IntentVerb::Recover),
];

/// Route markers, longest first. A sentence carrying one of these is *sending*
/// something to a seat, which is what makes the shared-phrase rule above fire.
const ROUTE_PHRASES: &[&str] = &[
    "转派给",
    "分配给",
    "移交给",
    "assign to",
    "route to",
    "delegate to",
    "派给",
    "assign",
    "route",
];

/// Object-type surface forms, longest first.
const OBJECT_PHRASES: &[(&str, IntentObjectType)] = &[
    ("任务单", IntentObjectType::Dispatch),
    ("调度单", IntentObjectType::Dispatch),
    ("派个活", IntentObjectType::Dispatch),
    ("派单", IntentObjectType::Dispatch),
    ("派活", IntentObjectType::Dispatch),
    ("工作项", IntentObjectType::Workitem),
    ("交接", IntentObjectType::Handoff),
    ("移交", IntentObjectType::Handoff),
    ("评论", IntentObjectType::Comment),
    ("批注", IntentObjectType::Comment),
    ("留言", IntentObjectType::Comment),
    ("工作", IntentObjectType::Workitem),
    ("任务", IntentObjectType::Workitem),
    ("dispatch", IntentObjectType::Dispatch),
    ("workitem", IntentObjectType::Workitem),
    ("work item", IntentObjectType::Workitem),
    ("handoff", IntentObjectType::Handoff),
    ("comment", IntentObjectType::Comment),
];

/// Owner markers, longest first. `requires_role_verb` marks the markers that are
/// too common in prose to trust on their own: for those, the token after the
/// name must begin with a role verb, or the marker is not an owner marker at all.
const OWNER_MARKERS: &[(&str, bool)] = &[
    ("转派给", false),
    ("分配给", false),
    ("移交给", false),
    ("assign to", false),
    ("route to", false),
    ("delegate to", false),
    ("派给", false),
    ("给", true),
    ("让", true),
    ("由", true),
];

/// Words that may follow a named owner to confirm the marker really introduced
/// one (`给 nimbus 派个活` → `派`).
const ROLE_VERBS: &[&str] = &[
    "派",
    "做",
    "来",
    "去",
    "处理",
    "负责",
    "执行",
    "接上",
    "跟进",
    "review",
    "do",
    "handle",
    "own",
    "fix",
    "implement",
    "verify",
];

/// Precondition markers, longest first.
const PRECONDITION_MARKERS: &[&str] = &["等到", "once", "after", "待", "等"];

/// Where a precondition clause ends.
const PRECONDITION_ENDERS: &[&str] = &[
    "之后", "以后", "然后", "再", "吧", "呀", "吗", "。", "，", ",", ";", "！", "!",
];

/// Branch prefixes that make a token a branch reference.
const BRANCH_PREFIXES: &[&str] = &["packet/", "track/", "main/", "master/"];

/// Topic markers.
const TOPIC_MARKERS: &[&str] = &["关于", "主题", "topic:", "re:"];

// =============================================================================
// Matching engine
// =============================================================================

/// All non-overlapping phrase matches, longest first. Returns `(start, end,
/// value)` with byte offsets into `hay`.
fn maximal_matches<'a, V: Copy>(hay: &str, table: &'a [(&'a str, V)]) -> Vec<(usize, usize, V)> {
    let mut hits: Vec<(usize, usize, V)> = Vec::new();
    for (phrase, value) in table {
        for (start, _) in hay.match_indices(phrase) {
            if phrase.is_ascii() && !ascii_boundary_ok(hay, start, phrase.len()) {
                continue;
            }
            hits.push((start, start + phrase.len(), *value));
        }
    }
    // Longest first, then earliest, so the greedy accept below is deterministic.
    hits.sort_by(|a, b| (b.1 - b.0).cmp(&(a.1 - a.0)).then(a.0.cmp(&b.0)));
    let mut accepted: Vec<(usize, usize, V)> = Vec::new();
    for (start, end, value) in hits {
        let overlaps = accepted
            .iter()
            .any(|(a_start, a_end, _)| start < *a_end && *a_start < end);
        if !overlaps {
            accepted.push((start, end, value));
        }
    }
    accepted.sort_by_key(|(start, _, _)| *start);
    accepted
}

/// An ASCII phrase must not sit inside a longer latin token (`add` inside
/// `address`).
fn ascii_boundary_ok(hay: &str, start: usize, len: usize) -> bool {
    let before_ok = hay[..start]
        .chars()
        .next_back()
        .map(|c| !c.is_ascii_alphanumeric())
        .unwrap_or(true);
    let after_ok = hay[start + len..]
        .chars()
        .next()
        .map(|c| !c.is_ascii_alphanumeric())
        .unwrap_or(true);
    before_ok && after_ok
}

fn matched_verbs(hay: &str, suppress_object_overlaps: bool) -> Vec<IntentVerb> {
    let object_spans: Vec<(usize, usize)> = if suppress_object_overlaps {
        maximal_matches(hay, OBJECT_PHRASES)
            .into_iter()
            .map(|(start, end, _)| (start, end))
            .collect()
    } else {
        Vec::new()
    };
    maximal_matches(hay, VERB_PHRASES)
        .into_iter()
        .filter(|(start, end, _)| {
            !object_spans
                .iter()
                .any(|(o_start, o_end)| start < o_end && o_start < end)
        })
        .map(|(_, _, v)| v)
        .collect()
}

/// Does the sentence contain any of these phrases (word-boundary aware for
/// latin)?
fn any_phrase(hay: &str, phrases: &[&str]) -> bool {
    phrases.iter().any(|p| {
        hay.match_indices(p)
            .any(|(start, _)| !p.is_ascii() || ascii_boundary_ok(hay, start, p.len()))
    })
}

fn matched_objects(hay: &str) -> Vec<IntentObjectType> {
    maximal_matches(hay, OBJECT_PHRASES)
        .into_iter()
        .map(|(_, _, o)| o)
        .collect()
}

/// Deduplicate while preserving first-seen order.
fn distinct<V: PartialEq>(values: Vec<V>) -> Vec<V> {
    let mut out: Vec<V> = Vec::new();
    for v in values {
        if !out.contains(&v) {
            out.push(v);
        }
    }
    out
}

// =============================================================================
// Owner hint
// =============================================================================

/// Lift the owner as typed. Returns `None` when no marker introduces one — the
/// resolver then tries an exact seat-name match over the sentence's latin
/// tokens, and asks when that finds nothing.
///
/// **Earliest marker wins.** "给 nimbus 派个活,别派给 flux" names nimbus first
/// and flux second; taking the first marker in the sentence (rather than the
/// first in the table) keeps the operator's primary instruction. Ties break to
/// the longest phrase.
fn owner_hint(hay: &str) -> Option<String> {
    let mut markers = maximal_matches(hay, OWNER_MARKERS);
    markers.sort_by(|a, b| a.0.cmp(&b.0).then((b.1 - b.0).cmp(&(a.1 - a.0))));
    for (_, end, requires_role_verb) in markers {
        let rest = &hay[end..];
        let Some((name, consumed)) = read_name_token(rest) else {
            continue;
        };
        if requires_role_verb && !starts_with_role_verb(&rest[consumed..]) {
            continue;
        }
        return Some(name);
    }
    None
}

/// Read a name-shaped token: a latin token, or a short CJK run. Returns the
/// token and how many bytes it consumed (so the caller can inspect what
/// follows).
fn read_name_token(s: &str) -> Option<(String, usize)> {
    let mut chars = s.char_indices().peekable();
    // Skip whitespace and the particles that never belong to a name.
    while let Some(&(idx, ch)) = chars.peek() {
        if ch.is_whitespace() || matches!(ch, '的' | '了' | '吧' | '呀' | '一' | '个') {
            chars.next();
        } else {
            let _ = idx;
            break;
        }
    }
    let start = chars.peek().map(|&(idx, _)| idx)?;
    let mut end = start;
    let mut token = String::new();
    for (idx, ch) in chars {
        let name_char = ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.') || is_cjk(ch);
        if !name_char {
            break;
        }
        if is_cjk(ch) && token.chars().filter(|c| is_cjk(*c)).count() >= 6 {
            break;
        }
        token.push(ch);
        end = idx + ch.len_utf8();
    }
    if token.is_empty() {
        None
    } else {
        Some((token, end))
    }
}

fn starts_with_role_verb(s: &str) -> bool {
    let trimmed = s.trim_start();
    ROLE_VERBS.iter().any(|v| {
        if v.is_ascii() {
            let lower = trimmed.to_ascii_lowercase();
            lower.starts_with(v)
                && lower[v.len()..]
                    .chars()
                    .next()
                    .map(|c| !c.is_ascii_alphanumeric())
                    .unwrap_or(true)
        } else {
            trimmed.starts_with(v)
        }
    })
}

fn is_cjk(ch: char) -> bool {
    ('\u{4e00}'..='\u{9fff}').contains(&ch)
}

// =============================================================================
// Qualifiers — captured verbatim
// =============================================================================

fn qualifiers(hay: &str) -> IntentQualifiers {
    IntentQualifiers {
        topic: topic(hay),
        branch: branch(hay),
        precondition: precondition(hay),
        budget_hint: budget_hint(hay),
    }
}

fn topic(hay: &str) -> Option<String> {
    // Bracketed first: an explicit delimiter is the least ambiguous statement.
    for (open, close) in [("「", "」"), ("《", "》"), ("“", "”"), ("\"", "\"")] {
        if let Some(start) = hay.find(open) {
            let after = &hay[start + open.len()..];
            if let Some(end) = after.find(close) {
                let captured = after[..end].trim();
                if !captured.is_empty() {
                    return Some(captured.to_string());
                }
            }
        }
    }
    for marker in TOPIC_MARKERS {
        if let Some(start) = hay.find(marker) {
            let after = &hay[start + marker.len()..];
            let captured = after
                .trim_start()
                .chars()
                .take_while(|c| !matches!(c, '，' | ',' | '。' | ';' | '；' | ' '))
                .collect::<String>();
            let captured = captured.trim();
            if !captured.is_empty() {
                return Some(captured.to_string());
            }
        }
    }
    None
}

fn branch(hay: &str) -> Option<String> {
    for term in latin_terms(hay) {
        if BRANCH_PREFIXES.iter().any(|p| term.starts_with(p)) {
            return Some(term);
        }
    }
    for marker in ["分支", "branch"] {
        if let Some(start) = hay.find(marker) {
            let after = &hay[start + marker.len()..];
            if let Some((token, _)) = read_name_token(after) {
                return Some(token);
            }
        }
    }
    None
}

/// Capture a precondition clause verbatim. Empty and single-character clauses
/// are discarded: they are prose noise, not a precondition, and reporting them
/// would train the operator to dismiss the question.
fn precondition(hay: &str) -> Option<String> {
    for marker in PRECONDITION_MARKERS {
        let Some(start) = hay.find(marker) else {
            continue;
        };
        if marker.is_ascii() && !ascii_boundary_ok(hay, start, marker.len()) {
            continue;
        }
        let after = &hay[start + marker.len()..];
        let mut clause = String::new();
        for ch in after.chars() {
            let ends = PRECONDITION_ENDERS
                .iter()
                .any(|e| after[clause.len()..].starts_with(e));
            if ends {
                break;
            }
            clause.push(ch);
        }
        let clause = clause
            .trim()
            .trim_matches(|c: char| c.is_whitespace() || c == '。')
            .to_string();
        if clause.chars().count() >= 2 {
            return Some(clause);
        }
    }
    None
}

fn budget_hint(hay: &str) -> Option<String> {
    // `N tokens` / `N token` / `N k` / `N 预算`
    let bytes = hay.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit() {
            let start = i;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
            let after = hay[i..].trim_start();
            let unit = ["tokens", "token", "预算", "k", "K"].iter().find(|u| {
                after
                    .to_ascii_lowercase()
                    .starts_with(&u.to_ascii_lowercase())
                    && after[u.len()..]
                        .chars()
                        .next()
                        .map(|c| !c.is_ascii_alphanumeric() && !is_cjk(c))
                        .unwrap_or(true)
            });
            if let Some(unit) = unit {
                return Some(format!("{} {}", &hay[start..i], unit));
            }
        } else {
            i += 1;
        }
    }
    // `预算 N` / `budget N`
    for marker in ["预算为", "预算", "budget"] {
        if let Some(start) = hay.find(marker) {
            let after = hay[start + marker.len()..].trim_start();
            let digits: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
            if !digits.is_empty() {
                return Some(format!("{marker} {digits}"));
            }
        }
    }
    None
}

// =============================================================================
// latin tokens
// =============================================================================

/// Every latin token in the sentence, lowercased, deduplicated, sorted. Used
/// only for **exact** matching (seat names, seat ids, capability tags), never
/// for keyword scoring.
pub fn latin_terms(s: &str) -> Vec<String> {
    let mut out: BTreeSet<String> = BTreeSet::new();
    let mut current = String::new();
    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | '/') {
            current.push(ch.to_ascii_lowercase());
        } else if !current.is_empty() {
            let trimmed = current.trim_matches(|c| matches!(c, '-' | '.' | '/'));
            if !trimmed.is_empty() {
                out.insert(trimmed.to_string());
            }
            current.clear();
        }
    }
    if !current.is_empty() {
        let trimmed = current.trim_matches(|c| matches!(c, '-' | '.' | '/'));
        if !trimmed.is_empty() {
            out.insert(trimmed.to_string());
        }
    }
    out.into_iter().collect()
}

// =============================================================================
// Choice lists (what a clarification offers)
// =============================================================================

/// The five supported verbs with one-line examples (§3.3 row 1).
pub fn verb_choices() -> Vec<ClarificationChoice> {
    IntentVerb::ALL
        .iter()
        .map(|v| ClarificationChoice {
            value: v.as_str().to_string(),
            label: format!("{} — {}", v.as_str(), v.one_line_example()),
            why: "one of the five verbs the intent entry supports".to_string(),
        })
        .collect()
}

/// The four object types with one-line examples (§3.3 row 5).
pub fn object_choices() -> Vec<ClarificationChoice> {
    IntentObjectType::ALL
        .iter()
        .map(|o| ClarificationChoice {
            value: o.as_str().to_string(),
            label: format!("{} — {}", o.as_str(), o.one_line_example()),
            why: "one of the four object types an intent can name".to_string(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn parsed(input: &str) -> Intent {
        match parse(input) {
            IntentParse::Parsed(i) => i,
            other => panic!("expected Parsed, got {other:?}"),
        }
    }

    #[test]
    fn the_contract_example_parses_end_to_end() {
        let intent = parsed("给 Nimbus 派个活,把唤醒服务的硬底兜底接上");
        assert_eq!(intent.verb, IntentVerb::Create);
        assert_eq!(intent.object_type, Some(IntentObjectType::Dispatch));
        assert_eq!(intent.owner_hint.as_deref(), Some("Nimbus"));
        assert_eq!(intent.terms, vec!["nimbus".to_string()]);
    }

    #[test]
    fn a_why_question_parses_as_explain_and_needs_no_object() {
        let intent = parsed("我们为什么放弃了 passport.js?");
        assert_eq!(intent.verb, IntentVerb::Explain);
        assert_eq!(intent.object_type, None);
        assert!(intent.terms.contains(&"passport.js".to_string()));
    }

    #[test]
    fn route_markers_name_the_owner() {
        for input in [
            "把一个派单派给 flux",
            "分配给 onyx 一个工作项",
            "assign to mira a dispatch",
            "派给 onyx 派个活",
        ] {
            let intent = parsed(input);
            assert_eq!(intent.verb, IntentVerb::Route, "{input}");
            assert!(intent.owner_hint.is_some(), "{input}");
            assert!(intent.object_type.is_some(), "{input}");
        }
        assert_eq!(
            parsed("把一个派单派给 flux").owner_hint.as_deref(),
            Some("flux")
        );
        assert_eq!(
            parsed("assign to mira a dispatch").owner_hint.as_deref(),
            Some("mira")
        );
    }

    #[test]
    fn a_route_marker_makes_a_shared_phrase_the_object_not_the_verb() {
        // The stated disambiguation rule: with a route marker present, 派单 is
        // the object being routed, not a create verb.
        let routed = parsed("把一个派单派给 flux");
        assert_eq!(routed.verb, IntentVerb::Route);
        assert_eq!(routed.object_type, Some(IntentObjectType::Dispatch));
        // Without a route marker the same phrase is the create verb.
        let created = parsed("创建一个派单");
        assert_eq!(created.verb, IntentVerb::Create);
        assert_eq!(created.object_type, Some(IntentObjectType::Dispatch));
    }

    #[test]
    fn a_weak_owner_marker_without_a_role_verb_is_not_an_owner() {
        // "由于" must not be read as "由 <owner>".
        assert_eq!(parsed("创建一个派单").owner_hint, None);
        assert_eq!(parsed("派个活,由于预算很紧").owner_hint, None);
    }

    #[test]
    fn an_unsupported_verb_is_a_result_not_an_error() {
        match parse("把唤醒服务的硬底兜底接上") {
            IntentParse::UnsupportedVerb { supported, .. } => {
                assert_eq!(supported.len(), 5);
                assert!(supported[0].label.contains("create"));
            }
            other => panic!("expected UnsupportedVerb, got {other:?}"),
        }
    }

    #[test]
    fn two_unrelated_verbs_stop_and_ask_rather_than_picking_the_first() {
        match parse("恢复一下,然后派个活") {
            IntentParse::AmbiguousVerb { candidates, .. } => {
                assert_eq!(
                    candidates,
                    vec![IntentVerb::Recover, IntentVerb::Create],
                    "span order is deterministic"
                );
            }
            other => panic!("expected AmbiguousVerb, got {other:?}"),
        }
    }

    #[test]
    fn a_generic_create_verb_with_no_object_asks_which_kind() {
        match parse("创建一个东西") {
            IntentParse::ObjectTypeUnclear { verb, reason, .. } => {
                assert_eq!(verb, IntentVerb::Create);
                assert_eq!(reason, "no object type was stated");
            }
            other => panic!("expected ObjectTypeUnclear, got {other:?}"),
        }
    }

    #[test]
    fn maximal_munch_keeps_one_object_type_per_sentence() {
        // "任务单" must win over "任务"; otherwise this reads as two objects.
        assert_eq!(
            parsed("创建一个任务单").object_type,
            Some(IntentObjectType::Dispatch)
        );
        assert_eq!(
            parsed("创建一个工作项").object_type,
            Some(IntentObjectType::Workitem)
        );
    }

    #[test]
    fn latin_phrases_respect_word_boundaries() {
        // "add" inside "address" must not create a verb.
        assert!(matches!(
            parse("address the flaky test"),
            IntentParse::UnsupportedVerb { .. }
        ));
    }

    #[test]
    fn a_precondition_clause_is_captured_verbatim() {
        let intent = parsed("等 Flux 验证通过再给 nimbus 派个活");
        assert_eq!(
            intent.qualifiers.precondition.as_deref(),
            Some("Flux 验证通过")
        );
        assert_eq!(intent.owner_hint.as_deref(), Some("nimbus"));
    }

    #[test]
    fn a_precondition_naming_an_event_id_is_captured_whole() {
        let intent = parsed("等 ev-supervisor-intent-tools-landed 落地再派活");
        assert_eq!(
            intent.qualifiers.precondition.as_deref(),
            Some("ev-supervisor-intent-tools-landed 落地")
        );
    }

    #[test]
    fn branch_and_budget_hints_are_captured_verbatim() {
        let with_unit = parsed("在 packet/nimbus/intent 上派个活,预算 4096 tokens");
        assert_eq!(
            with_unit.qualifiers.branch.as_deref(),
            Some("packet/nimbus/intent")
        );
        assert_eq!(
            with_unit.qualifiers.budget_hint.as_deref(),
            Some("4096 tokens")
        );
        // With no unit word after the number, the 预算 marker is what fires.
        let bare = parsed("在 packet/nimbus/intent 上派个活,预算 4096");
        assert_eq!(bare.qualifiers.budget_hint.as_deref(), Some("预算 4096"));
    }

    #[test]
    fn an_empty_intent_is_unparsed_not_a_guess() {
        assert!(matches!(parse("   "), IntentParse::Unparsed { .. }));
    }

    #[test]
    fn english_intents_parse_too() {
        let intent = parsed("create a dispatch for flux to verify the gate");
        assert_eq!(intent.verb, IntentVerb::Create);
        assert_eq!(intent.object_type, Some(IntentObjectType::Dispatch));
        // No marker introduced an owner, so the parser leaves the hint unset and
        // records the token; resolution is the planner's job, by exact match.
        assert_eq!(intent.owner_hint, None);
        assert!(intent.terms.contains(&"flux".to_string()));
    }

    #[test]
    fn an_exact_seat_token_resolves_even_without_a_marker() {
        // The planner, not the parser, does the exact seat match.
        let out = crate::intent::plan::plan(
            &crate::intent::tests_support::facts(),
            "create a dispatch for flux to verify the gate",
            "human:supervisor",
        );
        match out {
            crate::intent::plan::IntentPlan::Proposal(p) => {
                assert_eq!(p.ownership.seat_id, "seat-flux-001");
            }
            other => panic!("expected a proposal, got {other:?}"),
        }
    }

    #[test]
    fn every_domain_round_trips_through_its_wire_spelling() {
        for v in IntentVerb::ALL {
            assert_eq!(IntentVerb::from_str(v.as_str()).unwrap(), v);
            assert_eq!(v.to_string(), v.as_str());
        }
        for o in IntentObjectType::ALL {
            assert_eq!(IntentObjectType::from_str(o.as_str()).unwrap(), o);
        }
        for k in DepKind::ALL {
            assert_eq!(DepKind::from_str(k.as_str()).unwrap(), k);
        }
        let err = IntentVerb::from_str("delegate").unwrap_err().to_string();
        assert!(err.contains("verb"), "{err}");
        assert!(err.contains("never defaulted"), "{err}");
    }

    use crate::intent::schema::DepKind;
}
