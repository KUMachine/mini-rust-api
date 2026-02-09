/// UserId value object - represents a unique user identifier
#[derive(Clone, Copy, Debug)]
pub struct UserId(i32);

impl UserId {
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
