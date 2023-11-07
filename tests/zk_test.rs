/// docker run --name my-zookeeper -d -p 2181:2181 zookeeper
#[cfg(feature = "zookeeper")]
mod zookeeper {
	use std::time::Instant;

	use chrono::Duration;
	use dist_lock::core::DistLock;
	use dist_lock::core::LockConfig;
	use dist_lock::core::Lockable;
	use dist_lock::error::LockResult;
	use dist_lock::provider::ZookeeperDriver;
	use zookeeper::Acl;
	use zookeeper::CreateMode;
	use zookeeper::Watcher;
	use zookeeper::ZooKeeper;

	#[test]
	fn test_lock() -> LockResult<()> {
		let zk_client =
			ZooKeeper::connect("127.0.0.1:2181", core::time::Duration::from_secs(60), MyWatcher)?;
		let lock_name: String = "zk_lock".to_string();
		let driver = ZookeeperDriver::new(Some("/parent".to_owned()), &zk_client)?;
		let config = LockConfig::new(lock_name, Duration::seconds(0), Duration::seconds(10));
		let dist_lock = DistLock::new(config, driver);
		check_lock(&dist_lock)
	}

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

	#[test]
	fn test_zk_connected() -> LockResult<()> {
		use core::time::Duration;

		let path = "/parent/zk_lock";
		let zk_client = ZooKeeper::connect("127.0.0.1:2181", Duration::from_secs(60), MyWatcher)?;
		if zk_client.exists(path, false)?.is_none() {
			zk_client.create(
				path,
				"data".as_bytes().to_vec(),
				Acl::open_unsafe().clone(),
				CreateMode::Persistent,
			)?;
		}

		println!("{:?}", zk_client.get_data(path, false)?);
		Ok(())
	}

	struct MyWatcher;

	impl Watcher for MyWatcher {
		fn handle(&self, event: zookeeper::WatchedEvent) {
			println!("{:?}", event.path);
		}
	}
}
