// Knowledge read model (overnight step 4) — the topic tree and the
// memories-by-topic card list, exposed where the UI can call them.
//
// PURE READS: these commands write nothing and append no event. They are
// interface queries for a panel, not a recall — `recall` keeps its
// `MemoryRecalled` audit, and a panel that mutates what it renders would be a
// lie about who read what.
//
// Both commands delegate to the SAME `seatloom_core` repository functions the
// MCP wrappers (`get_knowledge_tree` / `get_topic_memories`) call, so the two
// surfaces cannot drift. Scope is hard: project + topic, never an all-projects
// read.

use seatloom_core::objects::memory::MemoryLayer;
use tauri::State;

use crate::dto::{
    KnowledgeMemoryCardDto, KnowledgeTopicChainEntryDto, KnowledgeTopicMemoriesDto,
    KnowledgeTopicNodeDto, KnowledgeTopicTreeDto,
};
use crate::state::AppState;

/// The hard ceiling on one page of memory cards. The token budget belongs to the
/// caller; this only stops an unbounded page.
const TOPIC_MEMORIES_LIMIT_MAX: i64 = 200;
/// The default page size.
const TOPIC_MEMORIES_LIMIT_DEFAULT: i64 = 50;

/// The knowledge topic tree for a project: every topic with its parent and its
/// ACTIVE, not-superseded memory count. An empty project is an empty tree, not
/// an error.
#[tauri::command]
pub async fn cmd_get_knowledge_tree(
    state: State<'_, AppState>,
    project_id: Option<String>,
) -> Result<KnowledgeTopicTreeDto, String> {
    let project = project_id.unwrap_or_else(|| state.default_project_id.clone());
    let nodes = state
        .db
        .knowledge_topic_tree(&project)
        .await
        .map_err(|e| e.to_string())?;
    Ok(KnowledgeTopicTreeDto {
        project_id: project,
        topics: nodes.into_iter().map(KnowledgeTopicNodeDto::from).collect(),
    })
}

/// The memories of one topic, at the requested disclosure depth, with the
/// topic's ancestor chain (a memory is never returned out of its context).
///
/// `depth` is `l0` (default, the ~100-token summary), `l1` (the ~2K overview) or
/// `l2` (the evidence pointer). An unknown depth is a typed refusal, never a
/// serde fallback.
#[tauri::command]
pub async fn cmd_get_topic_memories(
    state: State<'_, AppState>,
    topic_id: String,
    depth: Option<String>,
    include_superseded: Option<bool>,
    limit: Option<i64>,
) -> Result<KnowledgeTopicMemoriesDto, String> {
    let depth = match depth.as_deref() {
        None | Some("l0") => MemoryLayer::L0,
        Some("l1") => MemoryLayer::L1,
        Some("l2") => MemoryLayer::L2,
        Some(other) => {
            return Err(format!(
                "depth {other:?} is not one of l0 | l1 | l2 (hand-written FromStr; an unknown value is never defaulted)"
            ));
        }
    };
    let limit = limit.unwrap_or(TOPIC_MEMORIES_LIMIT_DEFAULT);
    if !(1..=TOPIC_MEMORIES_LIMIT_MAX).contains(&limit) {
        return Err(format!(
            "limit {limit} is outside 1..={TOPIC_MEMORIES_LIMIT_MAX}"
        ));
    }
    let chain = state
        .db
        .topic_chain_ancestors_first(&topic_id)
        .await
        .map_err(|e| e.to_string())?;
    if chain.is_empty() {
        return Err(format!("topic {topic_id:?} not found"));
    }
    let cards = state
        .db
        .topic_memories(&topic_id, depth, include_superseded.unwrap_or(false), limit)
        .await
        .map_err(|e| e.to_string())?;
    Ok(KnowledgeTopicMemoriesDto {
        topic_id,
        depth: depth.as_str().to_string(),
        topic_chain: chain
            .into_iter()
            .map(|t| KnowledgeTopicChainEntryDto {
                topic_id: t.id,
                title: t.title,
            })
            .collect(),
        memories: cards
            .into_iter()
            .map(KnowledgeMemoryCardDto::from)
            .collect(),
    })
}
