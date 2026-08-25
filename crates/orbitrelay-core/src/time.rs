//! UTC timestamps shared by OrbitRelay server crates.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use time::{OffsetDateTime, UtcOffset};

use crate::CoreError;

/// An immutable point in time that is always represented with a UTC offset.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Timestamp(OffsetDateTime);

impl Timestamp {
    /// Returns the current UTC time.
    #[must_use]
    pub fn now_utc() -> Self {
        Self(OffsetDateTime::now_utc())
    }

    /// Creates a UTC timestamp from Unix seconds.
    pub fn from_unix_timestamp(seconds: i64) -> Result<Self, CoreError> {
        Ok(Self(
            OffsetDateTime::from_unix_timestamp(seconds)?.to_offset(UtcOffset::UTC),
        ))
    }

    /// Returns the timestamp as Unix seconds for an explicit boundary conversion.
    #[must_use]
    pub fn unix_timestamp(&self) -> i64 {
        self.0.unix_timestamp()
    }
}

impl Serialize for Timestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Timestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = OffsetDateTime::deserialize(deserializer)?;
        Ok(Self(value.to_offset(UtcOffset::UTC)))
    }
}

#[cfg(test)]
mod tests {
    use super::Timestamp;

    #[test]
    fn creates_current_time() {
        let before = Timestamp::now_utc().unix_timestamp();
        let current = Timestamp::now_utc().unix_timestamp();
        let after = Timestamp::now_utc().unix_timestamp();

        assert!(before <= current);
        assert!(current <= after);
    }

    #[test]
    fn converts_unix_timestamp() {
        let timestamp = Timestamp::from_unix_timestamp(1_700_000_000)
            .expect("known Unix timestamp should be valid");

        assert_eq!(timestamp.unix_timestamp(), 1_700_000_000);
    }

    #[test]
    fn serializes_and_deserializes_as_utc() {
        let timestamp = Timestamp::from_unix_timestamp(1_700_000_000)
            .expect("known Unix timestamp should be valid");
        let encoded = serde_json::to_string(&timestamp).expect("timestamp should serialize");
        let decoded: Timestamp =
            serde_json::from_str(&encoded).expect("timestamp should deserialize");

        assert_eq!(timestamp, decoded);
        assert_eq!(decoded.unix_timestamp(), 1_700_000_000);
    }
}
