pub type LockResult<T> = core::result::Result<T, crate::error::LockError>;

#[derive(Debug, thiserror::Error)]
pub enum LockError {
	#[cfg(feature = "redis")]
	#[error("Redis error: {0}")]
	RedisError(#[from] redis::RedisError),

	#[cfg(feature = "r2d2")]
	#[error("R2d2 error: {0}")]
	R2d2Error(#[from] r2d2::Error),

	#[cfg(feature = "diesel")]
	#[error("Diesel error: {0}")]
	DieselError(#[from] diesel::result::Error),

	#[cfg(feature = "diesel")]
	#[error("Diesel connection error: {0}")]
	DieselConnError(#[from] diesel::result::ConnectionError),

	#[cfg(feature = "zookeeper")]
	#[error("Zookeeper error: {0}")]
	ZkError(#[from] ::zookeeper::ZkError),

	#[error("lock failed")]
	LockFailed,

	#[error("lock released")]
	LockReleased,

	#[error("invalid error: {0}")]
	InvalidLock(String),
}
