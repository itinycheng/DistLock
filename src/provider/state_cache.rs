use crate::core::LockState;

#[cfg(feature = "tokio")]
tokio::task_local! {
	static LOCK_STATE: LockState;
}

#[cfg(feature = "tokio")]
pub fn set_state(state: LockState) {
	LOCK_STATE.sync_scope(state, || {});
}
