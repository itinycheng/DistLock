use std::fmt::Display;

use chrono::Utc;
use diesel::sql_types::BigInt;
use diesel::sql_types::VarChar;
use diesel::RunQueryDsl;
use gethostname::gethostname;

use crate::core::LockConfig;
use crate::core::LockState;
use crate::core::Lockable;
use crate::error::LockResult;

use super::help::sql_stmt::extend_lock_sql;
use super::help::sql_stmt::insert_lock_sql;
use super::help::sql_stmt::release_lock_sql;
use super::help::sql_stmt::update_lock_sql;

const LOCK_TABLE: &'static str = "dist_lock";

#[derive(Debug)]
pub struct DieselDriver<T> {
	name: String,
	table: String,
	transport: T,
}

impl<T> DieselDriver<T> {
	pub fn new<P>(lock_name: &String, table_prefix: Option<P>, transport: T) -> Self
	where
		P: Display,
	{
		DieselDriver {
			name: lock_name.to_owned(),
			table: match table_prefix {
				Some(prefix) => format!("{}_{}", prefix, LOCK_TABLE),
				None => LOCK_TABLE.to_owned(),
			},
			transport,
		}
	}

	pub fn name(&self) -> &String {
		&self.name
	}

	pub fn table(&self) -> &String {
		&self.table
	}

	pub fn transport(&self) -> &T {
		&self.transport
	}
}

macro_rules! impl_lockable_diesel {
	(
		$client: ty,
		$self: ident,
		$conn: expr
	) => {
		impl Lockable for DieselDriver<$client> {
			fn acquire_lock(&mut $self, config: &LockConfig) -> LockResult<LockState> {
				let now = Utc::now();
				let until = now + config.max_lock;

				let mut locked = match diesel::sql_query(insert_lock_sql(&$self.table))
				.bind::<VarChar, _>(&config.name)
				.bind::<BigInt, _>(until.timestamp_millis())
				.bind::<BigInt, _>(now.timestamp_millis())
				.bind::<VarChar, _>(gethostname().to_string_lossy())
				.execute($conn)
				{
					Ok(count)  => count > 0,
					Err(_) => false
				};

				if !locked {
					locked = diesel::sql_query(update_lock_sql(&$self.table))
						.bind::<BigInt, _>(until.timestamp_millis())
						.bind::<BigInt, _>(now.timestamp_millis())
						.bind::<VarChar, _>(gethostname().to_string_lossy())
						.bind::<VarChar, _>(&$self.name)
						.bind::<BigInt, _>(until.timestamp_millis())
						.execute($conn)? > 0;
				}

				Ok(LockState::new(locked, Utc::now()))
			}

			fn release_lock(&mut $self, config: &LockConfig, state: &LockState) -> LockResult<LockState> {
				let lock_until = config.lock_at_least_until(state.locked_at);
				diesel::sql_query(release_lock_sql(&$self.table))
					.bind::<BigInt, _>(lock_until.timestamp_millis())
					.bind::<VarChar, _>(&$self.name)
					.execute($conn)?;
				Ok(LockState::new(false, Utc::now()))
			}

			fn extend_lock(&mut $self, config: &LockConfig) -> LockResult<LockState> {
				let now = Utc::now();
				let until = now + config.max_lock;
				let count = diesel::sql_query(extend_lock_sql(&$self.table))
				.bind::<BigInt, _>(until.timestamp_millis())
				.bind::<VarChar, _>(&$self.name)
				.bind::<VarChar, _>(gethostname().to_string_lossy())
				.bind::<BigInt, _>(now.timestamp_millis())
				.execute($conn)?;
				Ok(LockState::new(count > 0, Utc::now()))
			}
		}
	};
}

#[cfg(feature = "diesel_sqlite")]
impl_lockable_diesel!(::diesel::SqliteConnection, self, &mut self.transport);
#[cfg(feature = "diesel_postgres")]
impl_lockable_diesel!(::diesel::PgConnection, self, &mut self.transport);
#[cfg(feature = "diesel_mysql")]
impl_lockable_diesel!(::diesel::MysqlConnection, self, &mut self.transport);
#[cfg(feature = "diesel_sqlite_r2d2")]
impl_lockable_diesel!(
	::r2d2::Pool<::diesel::r2d2::ConnectionManager<::diesel::SqliteConnection>>,
	self,
	&mut self.transport.get()?
);
#[cfg(feature = "diesel_postgres_r2d2")]
impl_lockable_diesel!(
	::r2d2::Pool<::diesel::r2d2::ConnectionManager<::diesel::PgConnection>>,
	self,
	&mut self.transport.get()?
);
#[cfg(feature = "diesel_mysql_r2d2")]
impl_lockable_diesel!(
	::r2d2::Pool<::diesel::r2d2::ConnectionManager<::diesel::MysqlConnection>>,
	self,
	&mut self.transport.get()?
);
