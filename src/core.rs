use async_trait::async_trait;
use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;
use futures::executor::block_on;

use crate::error::LockResult;

#[derive(Debug)]
pub struct DistLock<T: Lockable> {
	pub(super) config: LockConfig,
	pub(super) driver: T,
	pub(super) state: LockState,
	pub(super) create_at: DateTime<Utc>,
}

impl<T: Lockable> DistLock<T> {
	pub fn new(config: LockConfig, driver: T) -> Self {
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

	pub async fn acquire(&mut self) -> LockResult<bool> {
		let state = self.driver.acquire_lock(&self.config).await?;
		self.state = state;
		Ok(self.state.is_locked)
	}

	pub async fn release(&mut self) -> LockResult<()> {
		let state = self.driver.release_lock(&self.config, &self.state).await?;
		self.state = state;
		Ok(())
	}

	pub async fn extend(&mut self) -> LockResult<bool> {
		let state = self.driver.extend_lock(&self.config).await?;
		self.state = state;
		Ok(self.state.is_locked)
	}
}

#[derive(Debug, Clone)]
pub struct LockConfig {
	pub(super) name: String,
	pub(super) min_lock: Duration,
	pub(super) max_lock: Duration,
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

#[derive(Debug, Default, Clone)]
pub struct LockState {
	pub(super) is_locked: bool,
	pub(super) locked_at: DateTime<Utc>,
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
pub trait Lockable {
	async fn acquire_lock(&self, config: &LockConfig) -> LockResult<LockState>;

	async fn release_lock(&self, config: &LockConfig, state: &LockState) -> LockResult<LockState>;

	async fn extend_lock(&self, config: &LockConfig) -> LockResult<LockState>;
}

impl<T: Lockable> Drop for DistLock<T> {
	fn drop(&mut self) {
		if self.state.is_locked {
			_ = block_on(self.release());
		}
	}
}
