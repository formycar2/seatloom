use serde::{Deserialize, Serialize};

macro_rules! define_id {
    ($name:ident, $prefix:literal) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(String);

        impl $name {
            pub fn new() -> Self {
                Self(format!("{}-{}", $prefix, nanoid::nanoid!(8)))
            }
            pub fn as_str(&self) -> &str { &self.0 }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

define_id!(SeatId, "seat");
define_id!(SessionId, "ses");
define_id!(WorkItemId, "wi");
define_id!(ArtifactId, "ar");
define_id!(HandoffId, "ho");
define_id!(PipelineId, "pl");
define_id!(PipelineRunId, "plrun");
define_id!(CheckpointId, "cp");
define_id!(DelegationId, "del");
define_id!(EventId, "ev");
