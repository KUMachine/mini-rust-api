//! Helpers for running CPU-heavy work off the async runtime

use crate::app::errors::{AppResult, ApplicationError};

/// Run a blocking closure (e.g. password hashing) on Tokio's blocking thread pool
pub(crate) async fn run_blocking<T, F>(f: F) -> AppResult<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| ApplicationError::Internal(format!("blocking task failed: {e}")))
}
