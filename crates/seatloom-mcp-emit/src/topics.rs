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
            TopicRejection::Db(e) => write!(f, "open_topic rejected: {e}"),
        }
    }
}

impl From<DbError> for TopicRejection {
    fn from(e: DbError) -> Self {
        TopicRejection::Db(e.to_string())
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
