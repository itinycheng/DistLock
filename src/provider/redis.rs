use chrono::Duration;
use chrono::Utc;

use gethostname::gethostname;

use r2d2::Pool;
use redis::Client;
use redis::Cmd;
use redis::Value;

use crate::core::LockConfig;
use crate::core::LockState;
use crate::core::Lockable;
use crate::error::LockResult;

const KEY_PREFIX: &str = "dist_lock";

#[derive(Debug)]
pub struct RedisDriver<'a, T> {
	key: String,
	transport: &'a T,
}

impl<'a, T> RedisDriver<'a, T> {
	pub fn new(lock_name: &String, transport: &'a T) -> Self {
		RedisDriver { key: format!("{}:{}", KEY_PREFIX, lock_name), transport }
	}

	#[inline]
	pub fn build_value() -> String {
		format!("{},{}", Utc::now().timestamp_millis(), gethostname().to_string_lossy())
	}

	#[inline]
	pub fn acquire_cmd(&self, config: &LockConfig) -> Cmd {
		let mut cmd = redis::cmd("SET");
		cmd.arg(&self.key)
			.arg(Self::build_value())
			.arg("NX")
			.arg("PX")
			.arg(config.max_lock.num_milliseconds() as usize);
		cmd
	}

	#[inline]
	pub fn release_cmd(&self, config: &LockConfig, remaining: Duration) -> Cmd {
		let mut cmd;
		if remaining.num_milliseconds() > 0 {
			cmd = redis::cmd("SET");
			cmd.arg(&self.key)
				.arg(Self::build_value())
				.arg("XX")
				.arg("PX")
				.arg((config.min_lock - remaining).num_milliseconds());
		} else {
			cmd = redis::cmd("DEL");
			cmd.arg(&self.key);
		};
		cmd
	}

	#[inline]
	pub fn extend_cmd(&self, config: &LockConfig) -> Cmd {
		let mut cmd = redis::cmd("SET");
		cmd.arg(&self.key)
			.arg(Self::build_value())
			.arg("XX")
			.arg("PX")
			.arg(config.max_lock.num_milliseconds() as usize);
		cmd
	}
}

macro_rules! impl_lockable_redis {
	($client:ty,
		$conn_fn_name: ident,
		$query_fn_name: ident,
		$($async: ident)?,
		$($await: tt)*
	) => {
		#[cfg_attr(any(feature = "tokio", feature = "async-std"), async_trait::async_trait)]
		impl<'a> Lockable for RedisDriver<'a, $client> {
			$($async)? fn acquire_lock(&self, config: &LockConfig) -> LockResult<LockState> {
				let mut conn = self.transport.$conn_fn_name()$($await)*?;
				let cmd = self.acquire_cmd(config);
				let value: Value = cmd.$query_fn_name(&mut conn)$($await)*?;

				Ok(LockState::new(matches!(value, Value::Okay), Utc::now()))
			}

			$($async)? fn release_lock(
				&self,
				config: &LockConfig,
				state: &LockState,
			) -> LockResult<LockState> {
				let elapsed = Utc::now() - state.locked_at;
				let remaining = config.min_lock - elapsed;
				let mut conn = self.transport.$conn_fn_name()$($await)*?;
				let cmd = self.release_cmd(config, remaining);
				cmd.$query_fn_name(&mut conn)$($await)*?;
				Ok(LockState::new(false, Utc::now()))
			}

			$($async)? fn extend_lock(&self, config: &LockConfig) -> LockResult<LockState> {
				let mut conn = self.transport.$conn_fn_name()$($await)*?;
				let cmd = self.extend_cmd(config);
				let value: Value = cmd.$query_fn_name(&mut conn)$($await)*?;
				Ok(LockState::new(matches!(value, Value::Okay), Utc::now()))
			}
		}
	}
}

#[cfg(any(feature = "tokio", feature = "async-std"))]
impl_lockable_redis!(::redis::cluster::ClusterClient, get_async_connection, query_async, async, .await);
#[cfg(any(feature = "tokio", feature = "async-std"))]
impl_lockable_redis!(::redis::Client, get_async_connection, query_async, async, .await);

#[cfg(not(any(feature = "tokio", feature = "async-std")))]
impl_lockable_redis!(::redis::Client, get_connection, query,,);
#[cfg(not(any(feature = "tokio", feature = "async-std")))]
impl_lockable_redis!(::redis::cluster::ClusterClient, get_connection, query,,);

#[cfg(feature = "r2d2")]
impl_lockable_redis!(::r2d2::Pool<::redis::cluster::ClusterClient> , get, query,,);
#[cfg(feature = "r2d2")]
impl_lockable_redis!(::r2d2::Pool<::redis::Client> , get, query,,);