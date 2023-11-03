//start a docker image: docker run -d --name my-redis -p 6379:6379 redis
#[cfg(feature = "redis")]
mod redis {
	use std::time::Instant;

	use chrono::Duration;
	use dist_lock::core::DistLock;
	use dist_lock::core::LockConfig;
	use dist_lock::core::Lockable;
	use dist_lock::error::LockResult;
	use dist_lock::provider::redis::RedisDriver;
	use redis::Client;

	#[cfg(feature = "redis_provider")]
	#[test]
	fn test_lock() -> LockResult<()> {
		let lock_name = "random_lock".to_string();
		let client = Client::open("redis://127.0.0.1:6379/")?;
		let driver = RedisDriver::new(&lock_name, &client);
		let config = LockConfig::new(lock_name, Duration::seconds(0), Duration::seconds(10));
		let dist_lock = DistLock::new(config, driver);
		check_lock(&dist_lock)
	}

	#[cfg(feature = "redis_r2d2_provider")]
	#[test]
	fn test_t2d2_lock() -> LockResult<()> {
		let client = Client::open("redis://127.0.0.1:6379/")?;
		let pool = r2d2::Pool::builder().max_size(2).build(client)?;
		let lock_name = "random_lock".to_string();
		let driver = RedisDriver::new(&lock_name, &pool);
		let config = LockConfig::new(lock_name, Duration::seconds(0), Duration::seconds(10));
		let dist_lock = DistLock::new(config, driver);
		check_lock(&dist_lock)
	}

	#[cfg(feature = "redis_tokio_provider")]
	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn test_tokio_lock() -> LockResult<()> {
		let lock_name = "random_lock".to_string();
		let client = Client::open("redis://127.0.0.1:6379/")?;
		let driver = RedisDriver::new(&lock_name, &client);
		let config = LockConfig::new(lock_name, Duration::seconds(0), Duration::seconds(10));
		let dist_lock = DistLock::new(config, driver);
		check_lock(&dist_lock).await
	}

	#[cfg(feature = "redis_async_std_provider")]
	#[async_std::test]
	async fn test_async_std_lock() -> LockResult<()> {
		let lock_name = "random_lock".to_string();
		let client = Client::open("redis://127.0.0.1:6379/")?;
		let driver = RedisDriver::new(&lock_name, &client);
		let config = LockConfig::new(lock_name, Duration::seconds(0), Duration::seconds(10));
		let dist_lock = DistLock::new(config, driver);
		check_lock(&dist_lock).await
	}

	#[cfg(feature = "async-std")]
	async fn check_lock<T: Lockable>(dist_lock: &DistLock<T>) -> LockResult<()> {
		let now = Instant::now();
		assert!(dist_lock.acquire().await?);
		async_std::task::sleep(core::time::Duration::from_secs(5)).await;
		assert!(dist_lock.extend().await?);
		async_std::task::sleep(core::time::Duration::from_secs(5)).await;
		dist_lock.release().await?;
		println!("{:?}", now.elapsed());
		Ok(())
	}

	#[cfg(feature = "tokio")]
	async fn check_lock<T: Lockable>(dist_lock: &DistLock<T>) -> LockResult<()> {
		let now = Instant::now();
		assert!(dist_lock.acquire().await?);
		tokio::time::sleep(core::time::Duration::from_secs(5)).await;
		assert!(dist_lock.extend().await?);
		tokio::time::sleep(core::time::Duration::from_secs(5)).await;
		dist_lock.release().await?;
		println!("{:?}", now.elapsed());
		Ok(())
	}

	#[cfg(not(any(feature = "tokio", feature = "async-std")))]
	fn check_lock<T: Lockable>(dist_lock: &DistLock<T>) -> LockResult<()> {
		use std::thread;
		let now = Instant::now();
		assert!(dist_lock.acquire()?);
		thread::sleep(core::time::Duration::from_secs(5));
		assert!(dist_lock.extend()?);
		thread::sleep(core::time::Duration::from_secs(5));
		dist_lock.release()?;
		println!("{:?}", now.elapsed());
		Ok(())
	}
}
