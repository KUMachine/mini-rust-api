use serde::{Deserialize, Serialize};

/// UserId value object - represents a unique user identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(i32);

impl UserId {
    /// Create a new UserId from an existing database ID
    pub fn from_i32(id: i32) -> Self {
        Self(id)
    }

    /// Get the inner i32 value
    pub fn value(&self) -> i32 {
        self.0
    }
}

impl From<i32> for UserId {
    fn from(id: i32) -> Self {
        Self(id)
    }
}

impl From<UserId> for i32 {
    fn from(id: UserId) -> Self {
        id.0
    }
}
