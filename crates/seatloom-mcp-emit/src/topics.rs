//! The `open_topic` tool — the governed creation of a recall scope (Aegis ruling
//! 2026-10-11).
//!
//! Why a tool and not an implicit create: `memories.topic_id` is NOT NULL with an
//! FK to `topics(id)`, and `topics` had zero rows and no code path that could
//! create one — so every `remember` failed on the foreign key. Letting `remember`
//! create the topic implicitly was ruled out: a typo would silently open a scope
//! nobody ever recalls from, and the schema itself demands a deliberate opener
//! (`topics.opened_by` is NOT NULL). Scope is an intentional act, not a side
//! effect of typing.
//!
//! The identity rule is the same one `get_next_dispatch` follows: `opened_by` is
//! resolved from `SEATLOOM_ACTOR_REF` server-side and is NEVER a parameter.

use serde::Deserialize;

use seatloom_core::db::repositories::DbError;
use seatloom_core::db::SeatloomDb;

use crate::dispatch::{resolve_project, resolve_seat, DispatchRejection};

/// `close_topic` — close a recall scope (024).
#[derive(Debug, Deserialize)]
pub struct CloseTopicArgs {
    pub topic_id: String,
}

/// `open_topic` — open (idempotently) a recall scope.
#[derive(Debug, Deserialize)]
pub struct OpenTopicArgs {
    pub topic_id: String,
    pub title: String,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub parent_topic_id: Option<String>,
}

/// Why an `open_topic` call was refused. Every refusal is typed and writes
/// nothing.
#[derive(Debug)]
pub enum TopicRejection {
    /// A required field is blank.
    Empty {
        field: &'static str,
    },
    /// The parent topic names no topic.
    ParentMissing {
        parent_topic_id: String,
    },
    /// The parent lives in another project — a scope tree does not straddle
    /// projects.
    ParentOtherProject {
        parent_topic_id: String,
        parent_project_id: String,
        project_id: String,
    },
    /// The proposed parent is the topic itself, or one of its own descendants —
    /// a cycle in the scope tree.
    Cycle {
        topic_id: String,
        parent_topic_id: String,
    },
    /// The project names no project.
    ProjectMissing {
        project_id: String,
    },
    /// `024`: the topic names no row. Closing an unknown id is a refusal, not a
    /// no-op — a no-op would let a caller believe a scope it never created is
    /// now closed.
    TopicMissing {
        topic_id: String,
    },
    /// `024`: the topic is not `open`. Reported separately from `TopicMissing`
    /// because the caller's next action differs: one is a typo, the other is a
    /// double close.
    NotOpen {
        topic_id: String,
        status: String,
    },
    Db(String),
}

impl std::fmt::Display for TopicRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TopicRejection::Empty { field } => write!(f, "{field} must not be empty"),
            TopicRejection::ParentMissing { parent_topic_id } => write!(
                f,
                "parent topic {parent_topic_id:?} names no topic — open the parent first; \\
                 a scope is never created implicitly"
            ),
            TopicRejection::ParentOtherProject {
                parent_topic_id,
                parent_project_id,
                project_id,
            } => write!(
                f,
                "parent topic {parent_topic_id:?} belongs to project {parent_project_id:?}, \\
                 not {project_id:?} — a scope tree does not straddle projects"
            ),
            TopicRejection::Cycle {
                topic_id,
                parent_topic_id,
            } => write!(
                f,
                "parent topic {parent_topic_id:?} would make {topic_id:?} its own ancestor — \\
                 the scope tree must stay a tree"
            ),
            TopicRejection::ProjectMissing { project_id } => {
                write!(
                    f,
                    "project {project_id:?} names no project — a scope is never fabricated"
                )
            }
            TopicRejection::TopicMissing { topic_id } => write!(
                f,
                "topic {topic_id:?} names no topic — closing an unknown scope is refused, \
                 never a no-op"
            ),
            TopicRejection::NotOpen { topic_id, status } => write!(
                f,
                "topic {topic_id:?} is {status:?}, not open — a close moves open -> closed and \
                 nothing else. Its memories are unaffected either way: a recall scope is a \
                 finding aid, not an access control."
            ),
            TopicRejection::Db(e) => write!(f, "topic rejected: {e}"),
        }
    }
}

impl From<DbError> for TopicRejection {
    fn from(e: DbError) -> Self {
        // The 024 refusals are mapped, not flattened into `Db`: a caller that
        // gets "topic rejected: topic \"x\" is \"closed\", not open" can act, while
        // "topic rejected: postgres error" tells it nothing. Every other DbError
        // still falls through to the generic arm — an unmapped error must stay
        // visible as what it is rather than be guessed into a typed shape.
        match e {
            DbError::TopicNotFound { id } => TopicRejection::TopicMissing { topic_id: id },
            DbError::TopicNotOpen { id, status } => TopicRejection::NotOpen {
                topic_id: id,
                status,
            },
            other => TopicRejection::Db(other.to_string()),
        }
    }
}

impl From<DispatchRejection> for TopicRejection {
    fn from(e: DispatchRejection) -> Self {
        match e {
            DispatchRejection::MissingActorRef => TopicRejection::Empty {
                field: "SEATLOOM_ACTOR_REF",
            },
            DispatchRejection::MissingProjectId => TopicRejection::Empty {
                field: "project_id",
            },
            other => TopicRejection::Db(other.to_string()),
        }
    }
}

/// `open_topic(topic_id, title, project_id?, parent_topic_id?)`.
///
/// Idempotent by id: re-opening an existing topic succeeds, changes nothing and
/// emits no event — an existing row is not a transition. Returns a one-line
/// summary naming what happened.
pub async fn open_topic(db: &SeatloomDb, args: OpenTopicArgs) -> Result<String, TopicRejection> {
    let identity = resolve_seat()?;
    let project_id = resolve_project(args.project_id)?;
    let topic_id = non_empty(&args.topic_id, "topic_id")?;
    let title = non_empty(&args.title, "title")?;

    // The project must exist — a scope is never fabricated.
    if db.get_project(&project_id).await?.is_none() {
        return Err(TopicRejection::ProjectMissing { project_id });
    }

    // The parent, when named, must exist, be in the same project, and not make
    // the new topic its own ancestor.
    if let Some(parent) = &args.parent_topic_id {
        let parent = non_empty(parent, "parent_topic_id")?;
        if parent == topic_id {
            return Err(TopicRejection::Cycle {
                topic_id,
                parent_topic_id: parent,
            });
        }
        let Some(parent_row) = db.get_topic(&parent).await? else {
            return Err(TopicRejection::ParentMissing {
                parent_topic_id: parent,
            });
        };
        if parent_row.project_id != project_id {
            return Err(TopicRejection::ParentOtherProject {
                parent_topic_id: parent,
                parent_project_id: parent_row.project_id,
                project_id,
            });
        }
        // A cycle: the parent's ancestor chain already contains the id we are
        // about to create (possible only if the tree was corrupted by direct
        // SQL — `topics` has no not-self CHECK — so it is checked, not assumed).
        let chain = db.topic_chain_inclusive(&parent).await?;
        if chain.iter().any(|t| t.id == topic_id) {
            return Err(TopicRejection::Cycle {
                topic_id,
                parent_topic_id: parent,
            });
        }
    }

    let parent_ref = args.parent_topic_id.as_deref().map(str::to_string);
    let (row, created) = db
        .open_topic(
            &topic_id,
            &project_id,
            &title,
            parent_ref.as_deref(),
            &identity.actor_ref,
        )
        .await?;

    if created {
        Ok(format!(
            "opened topic {topic_id} in project {project_id} (parent {}, opened by {}, status {}) — one TopicOpened event appended",
            parent_ref.as_deref().unwrap_or("none"),
            identity.actor_ref,
            row.status
        ))
    } else {
        Ok(format!(
            "topic {topic_id} is already open in project {} (parent {}) — unchanged, no event",
            row.project_id,
            row.parent_topic_id.as_deref().unwrap_or("none")
        ))
    }
}

/// `close_topic(topic_id)` — the 024 closure path.
///
/// NOT idempotent, and deliberately so. `open_topic` is idempotent because
/// re-opening an existing row is not a transition. A close IS a transition, and a
/// second close is a refusal that names the current state — answering "already
/// closed" as success would let a caller believe it had just changed something.
///
/// The memories are untouched, and that is the ruling rather than an omission:
/// `get_topic_memories` and the tree reads do not filter on topic status, so a
/// closed scope keeps answering exactly as before. Closing a recall scope is not
/// hiding its evidence.
///
/// `closed_by` is resolved from `SEATLOOM_ACTOR_REF` server-side, the same rule
/// `open_topic` follows — never a parameter.
pub async fn close_topic(db: &SeatloomDb, args: CloseTopicArgs) -> Result<String, TopicRejection> {
    let identity = resolve_seat()?;
    let topic_id = non_empty(&args.topic_id, "topic_id")?;

    let row = db.close_topic(&topic_id, &identity.actor_ref).await?;

    // The message names what changed and, because it is the question that
    // matters, what did NOT: the memories are untouched and still readable.
    Ok(format!(
        "closed topic {} in project {} (closed by {}) — status is now {}, one TopicClosed event \
         appended. Its memories are untouched and still readable: closing a recall scope is not \
         hiding its evidence.",
        row.id, row.project_id, identity.actor_ref, row.status
    ))
}

/// A required non-blank string field, trimmed.
fn non_empty(value: &str, field: &'static str) -> Result<String, TopicRejection> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(TopicRejection::Empty { field });
    }
    Ok(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The 024 refusals name the state, and the two "topic is wrong" cases are
    /// DISTINCT variants rather than one. Collapsing them would make "already
    /// closed" look like "never existed", and the caller's next action differs:
    /// one is a double close, the other is a typo.
    #[test]
    fn close_refusals_distinguish_missing_from_not_open() {
        let missing = TopicRejection::TopicMissing {
            topic_id: "t".to_string(),
        }
        .to_string();
        let not_open = TopicRejection::NotOpen {
            topic_id: "t".to_string(),
            status: "closed".to_string(),
        }
        .to_string();
        assert!(missing.contains("names no topic"), "{missing}");
        assert!(not_open.contains("not open"), "{not_open}");
        assert_ne!(
            missing, not_open,
            "the two refusals must not be the same sentence"
        );
        // And the not-open message answers the question that matters, so a caller
        // does not have to guess whether closing hid the memories.
        assert!(
            not_open.contains("finding aid"),
            "the refusal should say the memories are unaffected: {not_open}"
        );
    }

    /// The refusals name the rule, so a caller can fix the call without
    /// guessing — and none of them is a raw database error.
    #[test]
    fn refusals_name_the_rule_broken() {
        assert!(TopicRejection::ParentMissing {
            parent_topic_id: "t".into()
        }
        .to_string()
        .contains("open the parent first"));
        assert!(TopicRejection::ProjectMissing {
            project_id: "p".into()
        }
        .to_string()
        .contains("never fabricated"));
        assert!(TopicRejection::Cycle {
            topic_id: "a".into(),
            parent_topic_id: "b".into()
        }
        .to_string()
        .contains("its own ancestor"));
        assert!(TopicRejection::ParentOtherProject {
            parent_topic_id: "t".into(),
            parent_project_id: "p1".into(),
            project_id: "p2".into(),
        }
        .to_string()
        .contains("straddle projects"));
        assert!(TopicRejection::Empty { field: "title" }
            .to_string()
            .contains("title"));
    }
}
