use chrono::DateTime;
use chrono::Utc;

#[cfg(feature = "thread")]
use core::cell::Cell;

use crate::provider::redis::RedisLock;

#[cfg(feature = "thread")]
thread_local! {
	static LOCKED: Cell<DateTime<Utc>> = Cell::new(Default::default());
}

#[cfg(feature = "tokio")]
tokio::task_local! {
	static LOCKED: DateTime<Utc>;
}

pub fn set_local(time: DateTime<Utc>) {
	#[cfg(feature = "thread")]
	LOCKED.with(|cell| cell.set(time));

	#[cfg(feature = "tokio")]
	LOCKED.sync_scope(time, || {});
}

pub fn get_local() -> DateTime<Utc> {
	#[cfg(feature = "thread")]
	let value = LOCKED.with(|cell| cell.get());

	#[cfg(feature = "tokio")]
	let value = LOCKED.get();
	value
}

impl<T> Drop for RedisLock<'_, T> {
	fn drop(&mut self) {
		set_local(Default::default());
	}
}
