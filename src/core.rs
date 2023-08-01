use async_trait::async_trait;
use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;

use crate::error::LockResult;

#[derive(Debug)]
pub struct DistLock<'a, T> {
	pub(super) config: LockConfig,
	pub(super) driver: &'a T,
	pub(super) state: LockState,
	pub(super) create_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct LockConfig {
	pub(super) name: String,
	pub(super) min_lock: Duration,
	pub(super) max_lock: Duration,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct LockState {
	pub(super) is_locked: bool,
	pub(super) locked_at: DateTime<Utc>,
}

impl<'a, T> DistLock<'a, T> {
	pub fn new(config: LockConfig, driver: &'a T) -> Self {
		DistLock { config, driver, state: Default::default(), create_at: Utc::now() }
	}

	pub fn driver(&self) -> &T {
		&self.driver
	}

	pub fn config(&self) -> &LockConfig {
		&self.config
	}

	pub fn state(&self) -> &LockState {
		&self.state
	}

	pub fn create_at(&self) -> &DateTime<Utc> {
		&self.create_at
	}
}

impl LockConfig {
	pub fn new(name: String, min_lock: Duration, max_lock: Duration) -> LockConfig {
		LockConfig { name, min_lock, max_lock }
	}

	pub fn name(&self) -> &String {
		&self.name
	}

	pub fn min_lock(&self) -> &Duration {
		&self.min_lock
	}

	pub fn max_lock(&self) -> &Duration {
		&self.max_lock
	}
}

impl LockState {
	pub const fn new(is_locked: bool, locked_at: DateTime<Utc>) -> LockState {
		Self { is_locked, locked_at }
	}

	pub fn is_lock(&self) -> bool {
		self.is_locked
	}

	pub fn lock_time(&self) -> DateTime<Utc> {
		self.locked_at
	}
}

#[async_trait]
pub trait Lockable<T>: Sized {
	async fn acquire(&mut self) -> LockResult<bool>;

	async fn release(&mut self) -> LockResult<()>;

	async fn extend(&mut self) -> LockResult<bool>;
}
