use std::time::Instant;

use chrono::Duration;
use dist_lock::core::DistLock;
use dist_lock::core::LockConfig;
use dist_lock::error::LockResult;
use dist_lock::provider::redis::RedisDriver;
use redis::Client;

#[cfg_attr(feature = "tokio", tokio::test)]
async fn test_lock() -> LockResult<()> {
	//start a docker image: docker run -d --name my-redis -p 6379:6379 redis
    let now = Instant::now();
	let lock_name = "random_lock".to_string();
	let client = Client::open("redis://127.0.0.1:6379/")?;
	let driver = RedisDriver::new(&lock_name, &client);
	let config = LockConfig::new(lock_name, Duration::seconds(0), Duration::seconds(10));
	let mut dist_lock = DistLock::new(config, driver);
	
    assert!(dist_lock.acquire().await?);
	tokio::time::sleep(core::time::Duration::from_secs(5)).await;
	assert!(dist_lock.extend().await?);
	tokio::time::sleep(core::time::Duration::from_secs(5)).await;
	dist_lock.release().await?;
    println!("{:?}", now.elapsed());
	Ok(())
}
