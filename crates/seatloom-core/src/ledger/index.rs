use std::collections::HashMap;

use crate::ledger::event::{CanonicalEvent, ObjectRef};

/// In-memory index rebuilt from events.jsonl on startup.
/// This is a cache/projection only — SQLite FTS5 remains the future persisted L1/L2 store.
pub struct LedgerIndex {
    events: Vec<CanonicalEvent>,
    by_event_id: HashMap<String, usize>,
    by_event_type: HashMap<String, Vec<usize>>,
    by_object_ref: HashMap<String, Vec<usize>>,
}

impl LedgerIndex {
    pub fn new() -> Self {
        Self {
            events: vec![],
            by_event_id: HashMap::new(),
            by_event_type: HashMap::new(),
            by_object_ref: HashMap::new(),
        }
    }

    pub fn rebuild_from(&mut self, events: Vec<CanonicalEvent>) {
        self.events = events;
        self.by_event_id.clear();
        self.by_event_type.clear();
        self.by_object_ref.clear();

        for (index, event) in self.events.iter().enumerate() {
            self.by_event_id
                .insert(event.event_id.as_str().to_string(), index);
            self.by_event_type
                .entry(format!("{:?}", event.event_type))
                .or_default()
                .push(index);

            for object_ref in &event.object_refs {
                self.by_object_ref
                    .entry(object_ref_key(object_ref))
                    .or_default()
                    .push(index);
            }
        }
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn get(&self, event_id: &str) -> Option<&CanonicalEvent> {
        self.by_event_id
            .get(event_id)
            .and_then(|index| self.events.get(*index))
    }

    pub fn events_for_type(&self, event_type: &str) -> Vec<&CanonicalEvent> {
        self.by_event_type
            .get(event_type)
            .into_iter()
            .flat_map(|indexes| indexes.iter().filter_map(|index| self.events.get(*index)))
            .collect()
    }

    pub fn events_for_object_ref(&self, object_ref: &str) -> Vec<&CanonicalEvent> {
        self.by_object_ref
            .get(object_ref)
            .into_iter()
            .flat_map(|indexes| indexes.iter().filter_map(|index| self.events.get(*index)))
            .collect()
    }
}

impl Default for LedgerIndex {
    fn default() -> Self {
        Self::new()
    }
}

fn object_ref_key(object_ref: &ObjectRef) -> String {
    match object_ref {
        ObjectRef::Seat(id) => id.as_str().to_string(),
        ObjectRef::Session(id) => id.as_str().to_string(),
        ObjectRef::WorkItem(id) => id.as_str().to_string(),
        ObjectRef::Artifact(id) => id.as_str().to_string(),
        ObjectRef::Handoff(id) => id.as_str().to_string(),
        ObjectRef::Delegation(id) => id.as_str().to_string(),
        ObjectRef::Pipeline(id) => id.as_str().to_string(),
        ObjectRef::PipelineRun(id) => id.as_str().to_string(),
        ObjectRef::Checkpoint(id) => id.as_str().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use crate::ledger::event::{CanonicalEvent, EventType, ObjectRef};
    use crate::objects::handoff::ActorRef;
    use crate::objects::id::{EventId, SeatId, SessionId, WorkItemId};

    use super::LedgerIndex;

    #[test]
    fn rebuilds_lookup_maps_from_seeded_events() {
        let seat_id = SeatId::new();
        let session_id = SessionId::new();
        let workitem_id = WorkItemId::new();
        let first_event_id = EventId::new();
        let second_event_id = EventId::new();

        let first_event = CanonicalEvent {
            event_id: first_event_id.clone(),
            event_type: EventType::SessionStarted,
            occurred_at: Utc.with_ymd_and_hms(2026, 4, 29, 0, 0, 0)
                .single()
                .expect("valid timestamp"),
            actor_ref: ActorRef::Seat(seat_id.clone()),
            object_refs: vec![ObjectRef::Session(session_id.clone())],
            evidence_refs: vec![],
            payload: None,
        };
        let second_event = CanonicalEvent {
            event_id: second_event_id.clone(),
            event_type: EventType::WorkItemCreated,
            occurred_at: Utc.with_ymd_and_hms(2026, 4, 29, 0, 1, 0)
                .single()
                .expect("valid timestamp"),
            actor_ref: ActorRef::Seat(seat_id),
            object_refs: vec![ObjectRef::WorkItem(workitem_id.clone())],
            evidence_refs: vec![],
            payload: None,
        };

        let mut index = LedgerIndex::new();
        index.rebuild_from(vec![first_event, second_event]);

        assert_eq!(index.len(), 2);
        assert!(index.get(first_event_id.as_str()).is_some());
        assert_eq!(index.events_for_type("SessionStarted").len(), 1);
        assert_eq!(index.events_for_object_ref(session_id.as_str()).len(), 1);
        assert_eq!(index.events_for_object_ref(workitem_id.as_str()).len(), 1);
        assert!(index.get(second_event_id.as_str()).is_some());
    }
}
