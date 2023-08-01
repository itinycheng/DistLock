pub mod core;
pub mod error;
pub mod provider;

#[cfg(test)]
mod tests {
	use chrono::Duration;
	use chrono::Utc;

	#[test]
	fn it_works() {
		let a = Duration::max_value();
		let _b = Utc::now() + a;
	}
}
