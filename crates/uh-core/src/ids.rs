//! Strongly-typed identifiers.
//!
//! Every entity in the system has its own ID type. This prevents
//! accidentally passing a `PlanId` where a `StepId` is expected.
//!
//! All IDs serialize as plain UUID strings via `#[serde(transparent)]`.

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

macro_rules! id_type {
    ($name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            #[must_use]
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

id_type!(PlanId);
id_type!(StepId);
id_type!(SessionId);
id_type!(SkillId);
id_type!(UserId);

#[derive(Debug, thiserror::Error)]
pub enum IdError {
    #[error("invalid id format")]
    Invalid,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_distinct_types() {
        let p = PlanId::new();
        let s = StepId::new();
        assert_ne!(p.to_string(), s.to_string());
    }

    #[test]
    fn ids_serialize_as_uuid_string() {
        let p = PlanId::new();
        let json = serde_json::to_string(&p).unwrap();
        // UUID string with quotes: 36 chars + 2 quotes = 38
        assert_eq!(json.len(), 38);
    }
}
