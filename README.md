# Distributed Lock

> DistLock use external store like Redis, Mysql, Zookeeper as coordinator.

## Usage

Take redis as an example, for more details, see: <https://github.com/itinycheng/DistLock/tree/master/tests>

```rust
use std::time::Instant;
use chrono::Duration;
use dist_lock::core::DistLock;
use dist_lock::core::LockConfig;
use dist_lock::core::Lockable;
use dist_lock::error::LockResult;
use dist_lock::provider::redis::RedisDriver;
use redis::Client;

let lock_name = "random_lock".to_string();
let client = Client::open("redis://127.0.0.1:6379/")?;
let driver = RedisDriver::new(&lock_name, &client);
let config = LockConfig::new(lock_name, Duration::seconds(0), Duration::seconds(10));
let dist_lock = DistLock::new(config, driver);

let now = Instant::now();
assert!(dist_lock.acquire().await?);
tokio::time::sleep(core::time::Duration::from_secs(5)).await;
assert!(dist_lock.extend().await?);
tokio::time::sleep(core::time::Duration::from_secs(5)).await;
dist_lock.release().await?;
println!("{:?}", now.elapsed());
```

OR

```rust

use dist_lock::error::LockResult;
use dist_lock_codegen::dist_lock;
use redis::Client;
use std::sync::OnceLock;

static CLIENT: OnceLock<Client> = OnceLock::new();

#[dist_lock(name = "random_lock", at_most = "10s", at_least="6s", transport(create_redis_conn()?))]
pub async fn test_macro() -> LockResult<()> {
    println!("{:?}", random_lock.state());
    Ok(())
}

fn create_redis_conn<'a>() -> LockResult<&'a Client> {
    Ok(CLIENT.get_or_try_init(|| {
        Client::open("redis://127.0.0.1:6379/")
    })?)
}
```

## Providers

- [redis](https://github.com/redis-rs/redis-rs)
- [diesel](https://github.com/diesel-rs/diesel)
- [zookeeper](https://github.com/bonifaido/rust-zookeeper)
