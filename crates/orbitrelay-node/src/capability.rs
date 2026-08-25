//! Extensible node capability identifiers.

use std::fmt;

use serde::{Deserialize, Serialize};

/// An extensible capability advertised by a node, such as `sync` or `storage`.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Capability(String);

impl Capability {
    /// Creates a capability without imposing deployment-specific naming rules.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the capability string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Capability {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::Capability;

    #[test]
    fn sorts_and_deduplicates_in_capability_sets() {
        let capabilities = [
            Capability::new("storage"),
            Capability::new("sync"),
            Capability::new("storage"),
            Capability::new("compute"),
        ]
        .into_iter()
        .collect::<BTreeSet<_>>();

        assert_eq!(
            capabilities
                .iter()
                .map(Capability::as_str)
                .collect::<Vec<_>>(),
            vec!["compute", "storage", "sync"]
        );
    }
}
