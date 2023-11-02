//start a docker image: docker run -d --name my-redis -p 6379:6379 redis
#[cfg(feature = "redis")]
mod redis {
	use std::time::Instant;

	use chrono::Duration;
	use dist_lock::core::DistLock;
	use dist_lock::core::LockConfig;
	use dist_lock::error::LockResult;
	use dist_lock::provider::redis::RedisDriver;
	use redis::Client;

	#[cfg(feature = "tokio")]
	#[tokio::test(flavor ="multi_thread", worker_threads = 2)]
	async fn test_tokio_lock() -> LockResult<()> {
		let now = Instant::now();
		let lock_name = "random_lock".to_string();
		let client = Client::open("redis://127.0.0.1:6379/")?;
		let driver = RedisDriver::new(&lock_name, &client);
		let config = LockConfig::new(lock_name, Duration::seconds(0), Duration::seconds(10));
		let dist_lock = DistLock::new(config, driver);

		assert!(dist_lock.acquire().await?);
		tokio::time::sleep(core::time::Duration::from_secs(5)).await;
		assert!(dist_lock.extend().await?);
		tokio::time::sleep(core::time::Duration::from_secs(5)).await;
		dist_lock.release().await?;
		println!("{:?}", now.elapsed());
		Ok(())
	}

	#[cfg(feature = "redis_provider")]
	#[test]
	fn test_lock() -> LockResult<()> {
		use std::thread;
		let now = Instant::now();
		let lock_name = "random_lock".to_string();
		let client = Client::open("redis://127.0.0.1:6379/")?;
		let driver = RedisDriver::new(&lock_name, &client);
		let config = LockConfig::new(lock_name, Duration::seconds(0), Duration::seconds(10));
		let dist_lock = DistLock::new(config, driver);

		assert!(dist_lock.acquire()?);
		thread::sleep(core::time::Duration::from_secs(5));
		assert!(dist_lock.extend()?);
		thread::sleep(core::time::Duration::from_secs(5));
		dist_lock.release()?;
		println!("{:?}", now.elapsed());
		Ok(())
	}
}
