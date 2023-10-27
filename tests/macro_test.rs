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
