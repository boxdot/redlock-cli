use redis::RedisError;
use redis::Value;
use uuid::Uuid;

use std::time::{Duration, Instant};

/// Acquired redlock from Redis.
///
/// Releases the lock on drop. If the lock is already released, the operation is no-op.
/// Can be cloned to release the lock from different contextes.
#[derive(Debug, Clone)]
pub struct Redlock {
    servers: Vec<String>,
    name: String,
    id: Uuid,
}

impl Redlock {
    pub fn try_lock<S>(servers: &[S], lock_name: &str, ttl: Duration) -> Option<Self>
    where
        S: AsRef<str> + ToString,
    {
        let start = Instant::now();
        let id = Uuid::new_v4();

        let num_locked = servers
            .iter()
            .filter(|server| try_lock_instance(server.as_ref(), lock_name, id, ttl) == Ok(true))
            .count();

        let quorum = servers.len() / 2 + 1;
        let has_quorum = quorum <= num_locked;
        if !has_quorum || ttl <= start.elapsed() {
            for server in servers {
                let _ = try_unlock_instance(server.as_ref(), lock_name, id);
            }
            return None;
        }

        Some(Self {
            servers: servers.iter().map(ToString::to_string).collect(),
            name: lock_name.to_string(),
            id,
        })
    }
}

impl Drop for Redlock {
    fn drop(&mut self) {
        for server in &self.servers {
            let _ = try_unlock_instance(server, &self.name, self.id);
        }
    }
}

fn try_lock_instance(
    server: &str,
    lock_name: &str,
    id: Uuid,
    ttl: Duration,
) -> Result<bool, RedisError> {
    let client = redis::Client::open(server)?;
    let mut connection = client.get_connection()?;
    let result: Value = redis::cmd("SET")
        .arg(lock_name)
        .arg(id.to_string())
        .arg("nx")
        .arg("px")
        .arg(ttl.as_millis() as u64)
        .query(&mut connection)?;
    Ok(result == Value::Okay)
}

fn try_unlock_instance(server: &str, lock_name: &str, id: Uuid) -> Result<bool, RedisError> {
    const UNLOCK_SCRIPT: &str = r"
        if redis.call('get',KEYS[1]) == ARGV[1] then
            return redis.call('del',KEYS[1])
        else
            return 0
        end";
    let client = redis::Client::open(server)?;
    let mut connection = client.get_connection()?;
    let script = redis::Script::new(UNLOCK_SCRIPT);
    let result: i32 = script
        .key(lock_name)
        .arg(id.to_string())
        .invoke(&mut connection)?;
    Ok(result == 1)
}
