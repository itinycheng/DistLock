use cfg_if::cfg_if;
use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;

use crate::error::LockResult;

cfg_if! {
	if #[cfg(any(feature = "tokio", feature = "async-std"))] {
		type StateCell = crossbeam_utils::atomic::AtomicCell<LockState>;

		trait StateOpr {
			fn set(&self, state: LockState);
			fn get(&self) -> LockState;
		}

		impl StateOpr for StateCell {
			fn set(&self, state: LockState) {
				self.store(state);
			}

			fn get(&self) -> LockState {
				self.load()
			}
		}
	} else {
		type StateCell = std::cell::Cell<LockState>;
	}
}

#[derive(Debug)]
pub struct DistLock<T: Lockable> {
	pub(super) config: LockConfig,
	pub(super) driver: T,
	pub(super) state: StateCell,
	pub(super) create_at: DateTime<Utc>,
}

macro_rules! impl_dist_lock {
	(
		$($async: ident)?,
		$($await: tt)*
	) => {
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

			pub fn state(&self) -> LockState {
				self.state.get()
			}

			pub fn create_at(&self) -> &DateTime<Utc> {
				&self.create_at
			}

			pub $($async)? fn acquire(&self) -> LockResult<bool> {
				if self.state.get().check_locked(&self.config) {
					return Ok(true);
				}

				let state = self.driver.acquire_lock(&self.config)$($await)*?;
				self.state.set(state);
				Ok(state.is_locked)
			}

			pub $($async)? fn release(&self) -> LockResult<()> {
				if !self.state.get().is_locked {
					return Ok(());
				}

				let state = self.driver.release_lock(&self.config, &self.state.get())$($await)*?;
				self.state.set(state);
				Ok(())
			}

			pub $($async)? fn extend(&self) -> LockResult<bool> {
				if !self.state.get().is_locked {
					return Ok(false);
				}

				let state = self.driver.extend_lock(&self.config)$($await)*?;
				self.state.set(state);
				Ok(state.is_locked)
			}
		}
	};
}

#[cfg(not(any(feature = "tokio", feature = "async-std")))]
impl_dist_lock!(,);

#[cfg(any(feature = "tokio", feature = "async-std"))]
impl_dist_lock!(async, .await);

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

	pub fn from_mills(name: String, min_lock: i64, max_lock: i64) -> LockConfig {
		Self::new(name, Duration::milliseconds(min_lock), Duration::milliseconds(max_lock))
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

	pub fn lock_at_least_until(&self, locked_at: DateTime<Utc>) -> DateTime<Utc> {
		let now = Utc::now();
		let min_lock_until = locked_at + self.min_lock;
		if min_lock_until > now {
			min_lock_until
		} else {
			now
		}
	}
}

#[derive(Debug, Default, Clone, Copy)]
pub struct LockState {
	pub(super) is_locked: bool,
	pub(super) locked_at: DateTime<Utc>,
}

impl LockState {
	pub const fn unlock() -> LockState {
		Self { is_locked: false, locked_at: DateTime::<Utc>::MIN_UTC }
	}
	pub const fn new(is_locked: bool, locked_at: DateTime<Utc>) -> LockState {
		Self { is_locked, locked_at }
	}

	pub fn check_locked(&self, config: &LockConfig) -> bool {
		if self.is_locked {
			let until = self.locked_at + config.max_lock;
			until > Utc::now()
		} else {
			false
		}
	}

	pub fn lock_time(&self) -> DateTime<Utc> {
		self.locked_at
	}
}

macro_rules! impl_lockable {
	($($async: ident)?) => {
		#[cfg_attr(any(feature = "tokio", feature = "async-std"), async_trait::async_trait)]
		pub trait Lockable {
			$($async)? fn acquire_lock(&self, config: &LockConfig) -> LockResult<LockState>;

			$($async)? fn release_lock(
				&self,
				config: &LockConfig,
				state: &LockState,
			) -> LockResult<LockState>;

			$($async)? fn extend_lock(&self, config: &LockConfig) -> LockResult<LockState>;
		}
	};
}

#[cfg(not(any(feature = "tokio", feature = "async-std")))]
impl_lockable!();

#[cfg(any(feature = "tokio", feature = "async-std"))]
impl_lockable!(async);

impl<T: Lockable> Drop for DistLock<T> {
	fn drop(&mut self) {
		if self.state.get().is_locked {
			#[cfg(feature = "tokio")]
			tokio::task::block_in_place(|| {
				let _ = tokio::runtime::Handle::current().block_on(self.release());
			});

			#[cfg(feature = "async-std")]
			let _ = async_std::task::block_on(self.release());

			#[cfg(not(any(feature = "tokio", feature = "async-std")))]
			let _ = self.release();
		}
	}
}
