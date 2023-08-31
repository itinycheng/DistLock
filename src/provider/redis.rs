use chrono::Utc;

use gethostname::gethostname;

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

	#[inline(always)]
	fn build_value() -> String {
		format!("{},{}", Utc::now().timestamp_millis(), gethostname().to_string_lossy())
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
			$($async)? fn acquire_lock(&mut self, config: &LockConfig) -> LockResult<LockState> {
				let mut conn = self.transport.$conn_fn_name()$($await)*?;
				let value: Value = redis::cmd("SET")
					.arg(&self.key)
					.arg(Self::build_value())
					.arg("NX")
					.arg("PX")
					.arg(config.max_lock.num_milliseconds() as usize)
					.$query_fn_name(&mut conn)$($await)*?;
				Ok(LockState::new(matches!(value, Value::Okay), Utc::now()))
			}

			$($async)? fn release_lock(
				&mut self,
				config: &LockConfig,
				state: &LockState,
			) -> LockResult<LockState> {
				let until = config.lock_at_least_until(state.locked_at);
				let mut conn = self.transport.$conn_fn_name()$($await)*?;
				let remaining = (until - Utc::now()).num_milliseconds();
				if remaining > 0 {
					redis::cmd("SET")
						.arg(&self.key)
						.arg(Self::build_value())
						.arg("XX")
						.arg("PX")
						.arg(remaining)
						.$query_fn_name(&mut conn)$($await)*?;
				} else {
					redis::cmd("DEL").arg(&self.key).$query_fn_name(&mut conn)$($await)*?;
				}
				Ok(LockState::new(false, Utc::now()))
			}

			$($async)? fn extend_lock(&mut self, config: &LockConfig) -> LockResult<LockState> {
				let mut conn = self.transport.$conn_fn_name()$($await)*?;
				let value: Value = redis::cmd("SET")
					.arg(&self.key)
					.arg(Self::build_value())
					.arg("XX")
					.arg("PX")
					.arg(config.max_lock.num_milliseconds() as usize)
					.$query_fn_name(&mut conn)$($await)*?;
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
