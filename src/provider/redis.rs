use async_trait::async_trait;
use chrono::Utc;

use gethostname::gethostname;
use redis::Client;
use redis::Value;

use crate::core::DistLock;
use crate::core::Lockable;
use crate::error::LockResult;

const KEY_PREFIX: &str = "dist_lock";

#[async_trait]
impl<'a> Lockable<Client> for DistLock<'a, Client> {
	async fn acquire(&mut self) -> LockResult<bool> {
		let mut conn = self.driver.get_async_connection().await?;
		let value: Value = redis::cmd("SET")
			.arg(build_key(&self.config.name))
			.arg(build_value())
			.arg("NX")
			.arg("PX")
			.arg(self.config.max_lock.num_milliseconds() as usize)
			.query_async(&mut conn)
			.await?;

		self.state.is_locked = matches!(value, Value::Okay);
		self.state.locked_at = Utc::now();
		Ok(self.state.is_locked)
	}

	async fn release(&mut self) -> LockResult<()> {
		let elapsed = Utc::now() - self.state.locked_at;
		let remaining = self.config.min_lock - elapsed;
		let mut conn = self.driver.get_async_connection().await?;
		if remaining.num_milliseconds() > 0 {
			redis::cmd("SET")
				.arg(build_key(&self.config.name))
				.arg(build_value())
				.arg("XX")
				.arg("PX")
				.arg((self.config.min_lock - remaining).num_milliseconds())
				.query_async(&mut conn)
				.await?
		} else {
			redis::cmd("DEL").arg(build_key(&self.config.name)).query_async(&mut conn).await?
		};

		self.state.is_locked = false;
		Ok(())
	}

	async fn extend(&mut self) -> LockResult<bool> {
		let mut conn = self.driver.get_async_connection().await?;
		let value: Value = redis::cmd("SET")
			.arg(build_key(&self.config.name))
			.arg(Utc::now().timestamp_millis())
			.arg("XX")
			.arg("PX")
			.arg(self.config.max_lock.num_milliseconds() as usize)
			.query_async(&mut conn)
			.await?;

		self.state.is_locked = matches!(value, Value::Okay);
		self.state.locked_at = Utc::now();
		Ok(self.state.is_locked)
	}
}

fn build_key(name: &String) -> String {
	format!("{}:{}", KEY_PREFIX, name)
}

fn build_value() -> String {
	format!("{},{}", Utc::now().timestamp_millis(), gethostname().to_string_lossy())
}
