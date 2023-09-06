use std::fmt::Display;

cfg_if::cfg_if! {
	if #[cfg(feature = "diesel_postgres")] {
		#[inline(always)]
		pub fn insert_lock_sql<T>(table_name: T) -> String
		where
			T: Display,
		{
			format!(
				"INSERT INTO {} (name, lock_until, locked_at, locked_by) VALUES ($1, $2, $3, $4)",
				table_name
			)
		}

		#[inline(always)]
		pub fn update_lock_sql<T>(table_name: T) -> String
		where
			T: Display,
		{
			format!(
				"UPDATE {} SET lock_until = $1, locked_at = $2, locked_by = $3 WHERE name = $4 AND lock_until <= $5",
				table_name)
		}

		#[inline(always)]
		pub fn release_lock_sql<T>(table_name: T) -> String
		where
			T: Display,
		{
			format!("UPDATE {} SET lock_until = $1 WHERE name = $2", table_name)
		}

		#[inline(always)]
		pub fn extend_lock_sql<T>(table_name: T) -> String
		where
			T: Display,
		{
			format!(
				"UPDATE {} SET lock_until = $1 WHERE name = $2 AND locked_by = $3 AND lock_until > $4",
				table_name
			)
		}
	} else if #[cfg(any(feature = "diesel_sqlite", feature = "diesel_mysql"))]{

		#[inline(always)]
		pub fn insert_lock_sql<T>(table_name: T) -> String
		where
			T: Display,
		{
			format!(
				"INSERT INTO {} (name, lock_until, locked_at, locked_by) VALUES (?, ?, ?, ?)",
				table_name
			)
		}

		#[inline(always)]
		pub fn update_lock_sql<T>(table_name: T) -> String
		where
			T: Display,
		{
			format!(
				"UPDATE {} SET lock_until = ?, locked_at = ?, locked_by = ? WHERE name = ? AND lock_until <= ?",
				table_name)
		}

		#[inline(always)]
		pub fn release_lock_sql<T>(table_name: T) -> String
		where
			T: Display,
		{
			format!("UPDATE {} SET lock_until = ? WHERE name = ?", table_name)
		}

		#[inline(always)]
		pub fn extend_lock_sql<T>(table_name: T) -> String
		where
			T: Display,
		{
			format!(
				"UPDATE {} SET lock_until = ? WHERE name = ? AND locked_by = ? AND lock_until > ?",
				table_name
			)
		}
	}

}
