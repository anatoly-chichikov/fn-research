pub mod brief;
pub mod pending;
pub mod processor;
pub mod provider;
pub mod result;
pub mod session;
pub mod task;

/// Maximum time to wait for research task completion in hours.
pub const TASK_TIMEOUT_HOURS: u64 = 10;

#[cfg(any(test, feature = "test-support"))]
pub mod ids;
