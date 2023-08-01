use crate::core::LockState;

#[cfg(feature = "tokio")]
tokio::task_local! {
	static LOCK_STATE: LockState;
}

pub fn set_state(state: LockState) {
	#[cfg(feature = "tokio")]
	LOCK_STATE.sync_scope(state, || {});
}

pub fn get_state() -> Option<LockState> {
	#[cfg(feature = "tokio")]
	let value = LOCK_STATE.try_with(|e| *e).ok();
	value
}
