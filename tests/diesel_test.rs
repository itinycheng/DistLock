// 1. mysql image: docker run -d --name my-mysql -p 3306:3306 -e MYSQL_ROOT_PASSWORD=123456 mysql
// postgres image: docker run --name some-postgres -e POSTGRES_PASSWORD=123456 -e
// POSTGRES_USER=postgres -d -p 5432:5432 postgres
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
	use chrono::Duration;
	use diesel::Connection;
	use dist_lock::core::DistLock;
	use dist_lock::core::LockConfig;
	use dist_lock::core::Lockable;
	use dist_lock::error::LockResult;
	use dist_lock::provider::diesel::DieselDriver;
	use std::thread;
	use std::time::Instant;

	#[cfg(feature = "diesel_mysql")]
	#[test]
	fn test_mysql_lock() -> LockResult<()> {
		use diesel::MysqlConnection;
		let db_url = "mysql://root:123456@127.0.0.1:3306/diesel_test";
		let conn = MysqlConnection::establish(db_url)?;
		let lock_name = "random_lock".to_string();
		let driver = DieselDriver::new(&lock_name, Some("t"), conn);
		let config = LockConfig::new(lock_name, Duration::seconds(0), Duration::seconds(10));
		let dist_lock = DistLock::new(config, driver);
		check_lock(&dist_lock)
	}

	#[cfg(feature = "diesel_mysql_r2d2")]
	#[test]
	fn test_mysql_r2d2_lock() -> LockResult<()> {
		use diesel::r2d2::ConnectionManager;
		use diesel::MysqlConnection;
		use r2d2::Pool;
		let db_url = "mysql://root:123456@127.0.0.1:3306/diesel_test";
		let manager = ConnectionManager::<MysqlConnection>::new(db_url);
		let pool = Pool::builder().max_size(1).test_on_check_out(true).build(manager)?;
		let lock_name = "random_lock".to_string();
		let driver = DieselDriver::new(&lock_name, Some("t"), pool);
		let config = LockConfig::new(lock_name, Duration::seconds(0), Duration::seconds(10));
		let dist_lock = DistLock::new(config, driver);
		check_lock(&dist_lock)
	}

	#[cfg(feature = "diesel_postgres")]
	#[test]
	fn test_postgres_lock() -> LockResult<()> {
		use diesel::PgConnection;
		let db_url = "postgres://postgres:123456@127.0.0.1:5432/diesel_test";
		let conn = PgConnection::establish(db_url)?;
		let lock_name = "random_lock".to_string();
		let driver = DieselDriver::new(&lock_name, Some("t"), conn);
		let config = LockConfig::new(lock_name, Duration::seconds(0), Duration::seconds(10));
		let dist_lock = DistLock::new(config, driver);
		check_lock(&dist_lock)
	}

	#[cfg(feature = "diesel_postgres_r2d2")]
	#[test]
	fn test_postgres_r2d2_lock() -> LockResult<()> {
		use diesel::r2d2::ConnectionManager;
		use diesel::PgConnection;
		use r2d2::Pool;
		let db_url = "postgres://postgres:123456@127.0.0.1:5432/diesel_test";
		let manager = ConnectionManager::<PgConnection>::new(db_url);
		let pool = Pool::builder().max_size(1).test_on_check_out(true).build(manager)?;
		let lock_name = "random_lock".to_string();
		let driver = DieselDriver::new(&lock_name, Some("t"), pool);
		let config = LockConfig::new(lock_name, Duration::seconds(0), Duration::seconds(10));
		let dist_lock = DistLock::new(config, driver);
		check_lock(&dist_lock)
	}

	#[cfg(feature = "diesel_sqlite")]
	#[test]
	fn test_sqlite_lock() -> LockResult<()> {
		use diesel::connection::SimpleConnection;
		use diesel::SqliteConnection;
		let db_url = "diesel_test.db";
		let mut conn = SqliteConnection::establish(db_url)?;
		conn.batch_execute(
			r"CREATE TABLE IF NOT EXISTS t_dist_lock(
			name VARCHAR(64) NOT NULL,
			lock_until BIGINT NOT NULL,
			locked_at BIGINT NOT NULL,
			locked_by VARCHAR(255) NOT NULL,
			PRIMARY KEY (name)
		);",
		)?;
		let lock_name = "random_lock".to_string();
		let driver = DieselDriver::new(&lock_name, Some("t"), conn);
		let config = LockConfig::new(lock_name, Duration::seconds(0), Duration::seconds(10));
		let dist_lock = DistLock::new(config, driver);
		check_lock(&dist_lock)
	}

	#[cfg(feature = "diesel_sqlite_r2d2")]
	#[test]
	fn test_sqlite_r2d2_lock() -> LockResult<()> {
		use diesel::connection::SimpleConnection;
		use diesel::r2d2::ConnectionManager;
		use diesel::SqliteConnection;
		use r2d2::Pool;
		let db_url = "diesel_test.db";
		let manager = ConnectionManager::<SqliteConnection>::new(db_url);
		let pool = Pool::builder().max_size(1).test_on_check_out(true).build(manager)?;
		pool.get()?.batch_execute(
			r"CREATE TABLE IF NOT EXISTS t_dist_lock(
			name VARCHAR(64) NOT NULL,
			lock_until BIGINT NOT NULL,
			locked_at BIGINT NOT NULL,
			locked_by VARCHAR(255) NOT NULL,
			PRIMARY KEY (name)
		);",
		)?;

		let lock_name = "random_lock".to_string();
		let driver = DieselDriver::new(&lock_name, Some("t"), pool);
		let config = LockConfig::new(lock_name, Duration::seconds(0), Duration::seconds(10));
		let dist_lock = DistLock::new(config, driver);
		check_lock(&dist_lock)
	}

	fn check_lock<T: Lockable>(dist_lock: &DistLock<T>) -> LockResult<()> {
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
