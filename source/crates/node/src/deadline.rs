//! The resident operation's immutable deadline, scoped to its own future.
//!
//! Task-local storage carries it through the existing operation handlers.
//! Network stages pass the absolute deadline explicitly. A separately spawned
//! task must deliberately inherit it or start its own background budget.

use std::future::Future;

use tokio::time::Instant;

tokio::task_local! {
    static BY: Instant;
}

pub(crate) fn current() -> Option<Instant> {
    BY.try_with(|by| *by).ok()
}

pub(crate) async fn scope<T>(by: Instant, future: impl Future<Output = T>) -> T {
    let by = current().map_or(by, |parent| parent.min(by));
    BY.scope(by, future).await
}
