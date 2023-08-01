use async_trait::async_trait;
use chrono::DateTime;
use chrono::Utc;
use redis::Client;
use redis::Value;

use crate::core::LockConfig;
use crate::core::Lockable;
use crate::error::LockResult;

const KEY_PREFIX: &str = "dist_lock";

#[derive(Debug)]
pub struct RedisLock<'a, T> {
	pub key: String,
	pub config: LockConfig,
	pub driver: &'a T,
	pub create_at: DateTime<Utc>,
	pub locked_at: Option<DateTime<Utc>>,
}

impl<'a, T> RedisLock<'a, T> {
	pub fn new(config: LockConfig, driver: &'a T) -> Self {
		RedisLock {
			key: format!("{}:{}", KEY_PREFIX, &config.name),
			config,
			driver,
			create_at: Utc::now(),
			locked_at: None,
		}
	}
}

#[async_trait]
impl<'a> Lockable<Client> for RedisLock<'a, Client> {
	async fn acquire(&mut self) -> LockResult<bool> {
		let mut conn = self.driver.get_async_connection().await?;
		let value: Value = redis::cmd("SET")
			.arg(&self.key)
			.arg(Utc::now().timestamp_millis())
			.arg("NX")
			.arg("PX")
			.arg(self.config.max_lock.num_milliseconds() as usize)
			.query_async(&mut conn)
			.await?;

		match value {
			Value::Okay => {
				self.locked_at = Some(Utc::now());
				Ok(true)
			}
			_ => Ok(false),
		}
	}

	async fn release(&self) -> LockResult<bool> {
		let now = Utc::now();
		let elapsed = now - self.locked_at.unwrap_or_default();
		let remaining = self.config.min_lock - elapsed;
		let mut conn = self.driver.get_async_connection().await?;
		let value: Value = if remaining.num_milliseconds() > 0 {
			redis::cmd("SET")
				.arg(&self.key)
				.arg(Utc::now().timestamp_millis())
				.arg("XX")
				.arg("PX")
				.arg((self.config.min_lock - remaining).num_milliseconds())
				.query_async(&mut conn)
				.await?
		} else {
			redis::cmd("DEL").arg(&self.key).query_async(&mut conn).await?
		};

		match value {
			Value::Okay => Ok(true),
			_ => Ok(false),
		}
	}

	async fn extend(&mut self) -> LockResult<bool> {
		todo!()
	}
}
