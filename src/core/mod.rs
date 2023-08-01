use async_trait::async_trait;
use chrono::Duration;

use crate::error::LockResult;

pub mod local_cache;

#[derive(Debug)]
pub struct LockConfig {
	pub name: String,
	pub min_lock: Duration,
	pub max_lock: Duration,
}

#[async_trait]
pub trait Lockable<T>: Sized {
	async fn acquire(&mut self) -> LockResult<bool>;

	async fn release(&self) -> LockResult<bool>;

	async fn extend(&mut self) -> LockResult<bool>;
}
