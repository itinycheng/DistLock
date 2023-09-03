// 1. start a docker image: docker run -d --name my-mysql -p 3306:3306 -e MYSQL_ROOT_PASSWORD=123456
//    mysql
// 2. create db: create database diesel_test;
// 3. create table:
// CREATE TABLE t_dist_lock(
//     name VARCHAR(64) NOT NULL,
//     lock_until BIGINT NOT NULL,
//     locked_at BIGINT NOT NULL,
//     locked_by VARCHAR(255) NOT NULL,
//     PRIMARY KEY (name)
// );

#[cfg(feature = "diesel")]
mod diesel {
	use std::thread;
	use std::time::Instant;

	use chrono::Duration;
	use diesel::Connection;
	use diesel::MysqlConnection;
	use dist_lock::core::DistLock;
	use dist_lock::core::LockConfig;
	use dist_lock::error::LockResult;
	use dist_lock::provider::diesel::DieselDriver;

	#[test]
	fn test_lock() -> LockResult<()> {
		let now = Instant::now();
		let db_url = "mysql://root:123456@127.0.0.1:3306/diesel_test";
		let conn = MysqlConnection::establish(db_url)?;
		let lock_name = "random_lock".to_string();
		let driver = DieselDriver::new(&lock_name, Some("t"), conn);
		let config = LockConfig::new(lock_name, Duration::seconds(0), Duration::seconds(10));
		let mut dist_lock = DistLock::new(config, driver);

		assert!(dist_lock.acquire()?);
		thread::sleep(core::time::Duration::from_secs(5));
		assert!(dist_lock.extend()?);
		thread::sleep(core::time::Duration::from_secs(5));
		dist_lock.release()?;
		println!("{:?}", now.elapsed());
		Ok(())
	}
}
