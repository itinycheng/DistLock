//! A distributed lock that can cross processes, the lock rely on a coordinator like
//! redis/mysql/zookeeper/etc. to store state.
//!
//! # Features
//!
//! - `redis_common`: Use `::redis::Client` or `::redis::cluster::ClusterClient` as driver.
//! - `redis_r2d2`: Enable r2d2 connection pool.
//! - `redis_tokio`: Async lock with tokio.
//! - `redis_async_std`: Async lock with async-std.
//! - `diesel_sqlite`: Enable diesel/sqlite.
//! - `diesel_postgres`: Enable diesel/postgres.
//! - `diesel_mysql`: Enable diesel/mysql.
//! - `diesel_sqlite_r2d2`: Enable diesel/sqlite and diesel/r2d2.
//! - `diesel_postgres_r2d2`: Enable diesel/postgres and diesel/r2d2.
//! - `diesel_mysql_r2d2`: Enable diesel/postgres and diesel/r2d2.
//! - `zookeeper`: Use zookeeper as state store backend.
//!
//! # Examples
//!
//! Add `dist_lock` dependency to `Cargo.toml`:
//! ```
//! [dependencies]
//! dist_lock = {version = "*", features = ["redis_common"]}
//! ```
//!
//! And then the code:
//! ```
//! use std::time::Instant;
//! use chrono::Duration;
//! use dist_lock::core::DistLock;
//! use dist_lock::core::LockConfig;
//! use dist_lock::core::Lockable;
//! use dist_lock::error::LockResult;
//! use dist_lock::provider::redis::RedisDriver;
//! use redis::Client;
//!
//! let lock_name = "random_lock".to_string();
//! let client = Client::open("redis://127.0.0.1:6379/")?;
//! let driver = RedisDriver::new(&lock_name, &client);
//! let config = LockConfig::new(lock_name, Duration::seconds(0), Duration::seconds(10));
//! let dist_lock = DistLock::new(config, driver);
//!
//! // acquire/extend/release lock.
//! dist_lock.acquire();
//! dist_lock.extend();
//! dist_lock.release();
//! ```
//!
//! OR
//!
//! ```
//! #![feature(once_cell_try)]
//!
//! use dist_lock::error::LockResult;
//! use dist_lock_codegen::dist_lock;
//! use redis::Client;
//! use std::sync::OnceLock;
//!
//! static CLIENT: OnceLock<Client> = OnceLock::new();
//!
//! #[dist_lock(name = "random_lock", at_most = "10s", at_least="6s", transport(create_redis_conn()?))]
//! pub fn test_macro() -> LockResult<()> {
//!     println!("{:?}", random_lock.state());
//!     Ok(())
//! }
//!
//! fn create_redis_conn<'a>() -> LockResult<&'a Client> {
//!     Ok(CLIENT.get_or_try_init(|| {
//!         Client::open("redis://127.0.0.1:6379/")
//!     })?)
//! }
//! ```
pub mod core;
pub mod error;
pub mod provider;

pub use dist_lock_codegen::dist_lock;

#[cfg(test)]
mod tests {
	use chrono::Duration;
	use chrono::Utc;

	#[test]
	fn it_works() {
		let a = Duration::max_value();
		let _b = Utc::now() + a;
	}
}
