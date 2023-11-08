use chrono::DateTime;
use chrono::Utc;
use zookeeper::Acl;
use zookeeper::CreateMode;
use zookeeper::ZooKeeper;

use crate::core::LockConfig;
use crate::core::LockState;
use crate::core::Lockable;
use crate::error::LockError;
use crate::error::LockResult;

const DEFAULT_PARENT_PATH: &str = "/dist_lock";

pub struct ZookeeperDriver<'a> {
	parent: String,
	transport: &'a ZooKeeper,
}

impl ZookeeperDriver<'_> {
	pub fn new(parent: Option<String>, transport: &ZooKeeper) -> LockResult<ZookeeperDriver<'_>> {
		let formatted = match parent {
			Some(par) => {
				if !par.starts_with('/') {
					return Err(LockError::InvalidLock(format!("invalid absolute path: {}", par)));
				}

				if par.ends_with('/') {
					par[..par.len() - 1].to_owned()
				} else {
					par
				}
			}
			None => DEFAULT_PARENT_PATH.to_owned(),
		};

		Ok(ZookeeperDriver { parent: formatted, transport })
	}

	pub fn path(&self, name: &str) -> String {
		format!("{}/{}", &self.parent, &name)
	}

	pub fn transport(&self) -> &ZooKeeper {
		self.transport
	}

	pub fn check_locked(&self, path: &str, config: &LockConfig) -> LockResult<bool> {
		if self.transport.exists(path, false)?.is_some() {
			let tuple = self.transport.get_data(path, false)?;

			let ts = i64::from_be_bytes(tuple.0.try_into().map_err(|_| {
				LockError::InvalidLock("can't parse zk data to timestamp".to_string())
			})?);
			let lock_time =
				DateTime::from_timestamp(ts / 1000, ((ts % 1000) * 1_000_000) as u32).ok_or(
					LockError::InvalidLock(format!("convert ts: {} to DateTime failed", ts)),
				)?;

			Ok(lock_time + config.max_lock > Utc::now())
		} else {
			Ok(false)
		}
	}

	pub fn create_zk_path(&self, path: &str) -> LockResult<()> {
		let parts = path.split('/').filter(|p| !p.is_empty()).collect::<Vec<_>>();
		let mut cur_path = String::new();
		for part in parts {
			cur_path.push('/');
			cur_path.push_str(part);

			if self.transport.exists(&cur_path, false)?.is_none() {
				self.transport.create(
					&cur_path,
					vec![],
					Acl::open_unsafe().clone(),
					CreateMode::Persistent,
				)?;
			}
		}

		Ok(())
	}
}

impl Lockable for ZookeeperDriver<'_> {
	fn acquire_lock(&self, config: &LockConfig) -> LockResult<LockState> {
		let path = self.path(&config.name);
		if !self.check_locked(&path, config)? {
			if self.transport.exists(&path, false)?.is_none() {
				self.create_zk_path(&path)?;
			}

			let now = Utc::now();
			let data = now.timestamp_millis().to_be_bytes().to_vec();
			let _ = self.transport.set_data(&path, data, None)?;
			Ok(LockState::new(true, now))
		} else {
			Ok(LockState::unlock())
		}
	}

	fn release_lock(&self, config: &LockConfig, state: &LockState) -> LockResult<LockState> {
		let path = self.path(&config.name);
		let at_least_until = config.lock_at_least_until(state.locked_at);
		if at_least_until > Utc::now() {
			let data = at_least_until.timestamp_millis().to_be_bytes().to_vec();
			let _ = self.transport.set_data(&path, data, None)?;
			Ok(*state)
		} else {
			if self.transport.exists(&path, false)?.is_some() {
				self.transport.delete(&path, None)?;
			}
			Ok(LockState::unlock())
		}
	}

	fn extend_lock(&self, config: &LockConfig) -> LockResult<LockState> {
		let path = self.path(&config.name);
		if self.check_locked(&path, config)? {
			let data = Utc::now().timestamp_millis().to_be_bytes().to_vec();
			let _ = self.transport.set_data(&path, data, None)?;
			Ok(LockState::new(true, Utc::now()))
		} else {
			Ok(LockState::unlock())
		}
	}
}
