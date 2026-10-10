//! The knowledge read tools (overnight step 4): `get_knowledge_tree` and
//! `get_topic_memories` — thin wrappers over the SAME `seatloom_core` repository
//! functions the Tauri commands call. One implementation, two surfaces.
//!
//! PURE READS: no writes, no events. These are interface queries for a panel,
//! not a recall — `recall` keeps its `MemoryRecalled` audit, and a panel that
//! mutates what it renders would be a lie about who read what.

use serde::Deserialize;
use serde_json::Value;

use seatloom_core::db::SeatloomDb;
use seatloom_core::objects::memory::MemoryLayer;

use crate::dispatch::{resolve_project, DispatchRejection};

/// The hard ceiling on one page of memory cards.
const LIMIT_MAX: i64 = 200;
/// The default page size.
const LIMIT_DEFAULT: i64 = 50;

/// `get_knowledge_tree` — the topic tree of a project.
#[derive(Debug, Deserialize)]
pub struct GetKnowledgeTreeArgs {
    #[serde(default)]
    pub project_id: Option<String>,
}

/// `get_topic_memories` — the memories of one topic, at a disclosure depth.
#[derive(Debug, Deserialize)]
pub struct GetTopicMemoriesArgs {
    pub topic_id: String,
    /// 'l0' (default) | 'l1' | 'l2'.
    #[serde(default)]
    pub depth: Option<String>,
    #[serde(default)]
    pub include_superseded: Option<bool>,
    #[serde(default)]
    pub limit: Option<i64>,
}

/// Why a knowledge read was refused.
#[derive(Debug)]
pub enum KnowledgeRejection {
    /// A string that is not in the closed domain it was parsed against.
    BadDomain {
        domain: &'static str,
        value: String,
        expected: &'static str,
    },
    /// The topic names no topic.
    TopicNotFound {
        topic_id: String,
    },
    /// The page size is outside the ceiling.
    LimitTooLarge {
        limit: i64,
    },
    Db(String),
}

impl std::fmt::Display for KnowledgeRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KnowledgeRejection::BadDomain { domain, value, expected } => write!(
                f,
                "{domain} {value:?} is not one of {expected} (hand-written FromStr; an unknown value is never defaulted)"
            ),
            KnowledgeRejection::TopicNotFound { topic_id } => {
                write!(f, "topic {topic_id:?} not found")
            }
            KnowledgeRejection::LimitTooLarge { limit } => {
                write!(f, "limit {limit} exceeds the hard ceiling {LIMIT_MAX}")
            }
            KnowledgeRejection::Db(e) => write!(f, "knowledge read rejected: {e}"),
        }
    }
}

impl From<seatloom_core::db::repositories::DbError> for KnowledgeRejection {
    fn from(e: seatloom_core::db::repositories::DbError) -> Self {
        KnowledgeRejection::Db(e.to_string())
    }
}

impl From<DispatchRejection> for KnowledgeRejection {
    fn from(e: DispatchRejection) -> Self {
        match e {
            DispatchRejection::MissingActorRef => KnowledgeRejection::BadDomain {
                domain: "SEATLOOM_ACTOR_REF",
                value: String::new(),
                expected: "the server must be launched per seat with its own actor ref",
            },
            DispatchRejection::MissingProjectId => KnowledgeRejection::BadDomain {
                domain: "project_id",
                value: String::new(),
                expected:
                    "pass project_id or set SEATLOOM_PROJECT_ID; a project is never fabricated",
            },
            other => KnowledgeRejection::Db(other.to_string()),
        }
    }
}

/// `get_knowledge_tree(project_id?)` — the topic tree with memory counts.
///
/// Pure read: no write, no event.
pub async fn get_knowledge_tree(
    db: &SeatloomDb,
    args: GetKnowledgeTreeArgs,
) -> Result<String, KnowledgeRejection> {
    let project_id = resolve_project(args.project_id)?;
    let nodes = db.knowledge_topic_tree(&project_id).await?;
    let out = serde_json::json!({
        "projectId": project_id,
        "topics": nodes
            .into_iter()
            .map(|n| serde_json::json!({
                "topicId": n.topic_id,
                "title": n.title,
                "status": n.status,
                "parentTopicId": n.parent_topic_id,
                "memoryCount": n.memory_count,
            }))
            .collect::<Vec<_>>(),
    });
    serde_json::to_string_pretty(&out).map_err(|e| KnowledgeRejection::Db(e.to_string()))
}

/// `get_topic_memories(topic_id, depth?, include_superseded?, limit?)` — the
/// memories of one topic at the requested depth, with the ancestor chain.
///
/// Pure read: no write, no event.
pub async fn get_topic_memories(
    db: &SeatloomDb,
    args: GetTopicMemoriesArgs,
) -> Result<String, KnowledgeRejection> {
    let depth = match args.depth.as_deref() {
        None | Some("l0") => MemoryLayer::L0,
        Some("l1") => MemoryLayer::L1,
        Some("l2") => MemoryLayer::L2,
        Some(other) => {
            return Err(KnowledgeRejection::BadDomain {
                domain: "depth",
                value: other.to_string(),
                expected: "l0 | l1 | l2",
            })
        }
    };
    let limit = args.limit.unwrap_or(LIMIT_DEFAULT);
    if !(1..=LIMIT_MAX).contains(&limit) {
        return Err(KnowledgeRejection::LimitTooLarge { limit });
    }
    let chain = db.topic_chain_ancestors_first(&args.topic_id).await?;
    if chain.is_empty() {
        return Err(KnowledgeRejection::TopicNotFound {
            topic_id: args.topic_id,
        });
    }
    let cards = db
        .topic_memories(
            &args.topic_id,
            depth,
            args.include_superseded.unwrap_or(false),
            limit,
        )
        .await?;
    let out = serde_json::json!({
        "topicId": args.topic_id,
        "depth": depth.as_str(),
        "topicChain": chain
            .into_iter()
            .map(|t| serde_json::json!({"topicId": t.id, "title": t.title}))
            .collect::<Vec<_>>(),
        "memories": cards
            .into_iter()
            .map(|c| {
                let mut v = serde_json::json!({
                    "memoryId": c.memory_id,
                    "topicId": c.topic_id,
                    "title": c.title,
                    "kind": c.kind,
                    "confidence": c.confidence,
                    "tier": c.tier,
                    "status": c.status,
                    "l0Summary": c.l0_summary,
                    "supersededBy": c.superseded_by,
                    "sourceKind": c.source_kind,
                    "sourceRef": c.source_ref,
                });
                if let Some(l1) = c.l1_overview {
                    v["l1Overview"] = Value::String(l1);
                }
                if let Some(p) = c.l2 {
                    v["l2"] = serde_json::json!({
                        "refKind": p.ref_kind,
                        "documentId": p.document_id,
                        "filePath": p.file_path,
                        "bodyDigest": p.body_digest,
                        "lineStart": p.line_start,
                        "lineEnd": p.line_end,
                    });
                }
                v
            })
            .collect::<Vec<_>>(),
    });
    serde_json::to_string_pretty(&out).map_err(|e| KnowledgeRejection::Db(e.to_string()))
}
