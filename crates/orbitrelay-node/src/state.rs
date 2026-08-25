//! Node lifecycle states.

use serde::{Deserialize, Serialize};

/// The externally visible lifecycle state of a node.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeState {
    /// The node is initializing and cannot yet accept normal work.
    Starting,
    /// The node is ready to provide its advertised capabilities.
    Ready,
    /// The node is finishing existing work and should receive no new work.
    Draining,
    /// The node is not available for work.
    Offline,
}

#[cfg(test)]
mod tests {
    use super::NodeState;

    #[test]
    fn serializes_states_as_snake_case() {
        assert_eq!(
            serde_json::to_string(&NodeState::Draining).expect("state should serialize"),
            "\"draining\""
        );
    }
}
