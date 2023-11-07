#[cfg(feature = "diesel_mysql")]
mod diesel_macro {

	use diesel::Connection;
	use diesel::MysqlConnection;
	use dist_lock::dist_lock;
	use dist_lock::error::LockResult;

	#[test]
	fn test_mysql_lock_macro() -> LockResult<()> {
		Ok(test_macro()?)
	}

	#[dist_lock(name = "test_lock", at_most = "15s", at_least="10s", transport(create_mysql_conn()?))]
	fn test_macro() -> LockResult<()> {
		Ok(())
	}

	fn create_mysql_conn() -> LockResult<MysqlConnection> {
		let db_url = "mysql://root:123456@127.0.0.1:3306/diesel_test";
		let conn = MysqlConnection::establish(db_url)?;
		Ok(conn)
	}
}

#[cfg(feature = "zookeeper")]
mod zk_macro {
	use std::sync::OnceLock;
	use std::time::Duration;

	use dist_lock::error::LockResult;
	use dist_lock_codegen::dist_lock;
	use zookeeper::Watcher;
	use zookeeper::ZooKeeper;

	static ZK: OnceLock<ZooKeeper> = OnceLock::new();

	#[test]
	fn test_zk_lock_macro() -> LockResult<()> {
		test_zk_macro()
	}

	#[dist_lock(
		name = "test_zk_macro",
		at_most = "15s",
		at_least = "10s",
		transport(create_zk_conn())
	)]
	fn test_zk_macro() -> LockResult<()> {
		println!("test zk macro");
		Ok(())
	}

	fn create_zk_conn<'a>() -> &'a ZooKeeper {
		ZK.get_or_init(|| {
			ZooKeeper::connect("127.0.0.1:2181", Duration::from_secs(60), ZkWatcher).unwrap()
		})
	}

	struct ZkWatcher;

	impl Watcher for ZkWatcher {
		fn handle(&self, event: zookeeper::WatchedEvent) {
			println!("{:?}", event.path);
		}
	}
}
