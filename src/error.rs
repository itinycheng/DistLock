pub type LockResult<T> = core::result::Result<T, crate::error::LockError>;

#[derive(Debug, thiserror::Error)]
pub enum LockError {
	#[error("Redis error: {0}")]
	RedisError(#[from] redis::RedisError),

	#[error("lock failed")]
	LockFailed,
}
