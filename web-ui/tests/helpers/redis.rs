//! TestRedis - Redis testing utilities for web-ui
//!
//! This module provides a `TestRedis` struct that creates an isolated Redis instance
//! for testing using Docker containers. It provides a clean, easy-to-use API for
//! interacting with Redis in tests.
//!
//! # Features
//! - Automatic Docker container management with testcontainers
//! - Full Redis API coverage (strings, hashes, lists, sets, counters)
//! - Test data seeding and cleanup utilities
//! - Compatible with the application's AppState
//!
//! # Usage
//!
//! ```rust
//! use helpers::redis::TestRedis;
//!
//! #[tokio::test]
//! async fn test_redis_functionality() {
//!     let redis = TestRedis::new().await;
//!
//!     // Use for AppState creation
//!     let redis_client = redis.get_client();
//!
//!     // Test Redis operations
//!     redis.set("key", "value").await.unwrap();
//!     let value = redis.get("key").await.unwrap();
//!     assert_eq!(value, Some("value".to_string()));
//!
//!     // Cleanup
//!     let _ = redis.cleanup().await;
//! }
//! ```
//!
//! # Safety
//! - Each TestRedis instance creates its own isolated Redis container
//! - TestRedis cannot be cloned (use separate instances for concurrent tests)
//! - Containers are automatically cleaned up when the instance is dropped

use once_cell::sync::Lazy;
use testcontainers::{Container, clients::Cli};
use testcontainers_modules::redis::Redis;

static DOCKER: Lazy<Cli> = Lazy::new(Cli::default);

pub struct TestRedis {
    pub client: redis::Client,
    pub connection_url: String,
    _container: Container<'static, Redis>,
}

impl TestRedis {
    pub async fn new() -> Self {
        let container = DOCKER.run(Redis);

        let port = container.get_host_port_ipv4(6379);
        let connection_url = format!("redis://127.0.0.1:{}", port);

        // Wait for Redis to be ready
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        // Create Redis client
        let client =
            redis::Client::open(connection_url.clone()).expect("Failed to create Redis client");

        // Test the connection
        let mut conn = client
            .get_connection()
            .expect("Failed to get Redis connection");
        let _: String = redis::cmd("PING")
            .query(&mut conn)
            .expect("Redis ping failed");

        Self {
            client,
            connection_url,
            _container: container,
        }
    }

    pub fn get_client(&self) -> redis::Client {
        self.client.clone()
    }

    pub async fn get_connection(&self) -> Result<redis::Connection, redis::RedisError> {
        self.client.get_connection()
    }

    pub async fn set(&self, key: &str, value: &str) -> Result<(), redis::RedisError> {
        let mut conn = self.client.get_connection()?;
        redis::cmd("SET").arg(key).arg(value).exec(&mut conn)?;
        Ok(())
    }

    pub async fn get(&self, key: &str) -> Result<Option<String>, redis::RedisError> {
        let mut conn = self.client.get_connection()?;
        redis::cmd("GET").arg(key).query(&mut conn)
    }

    pub async fn del(&self, key: &str) -> Result<(), redis::RedisError> {
        let mut conn = self.client.get_connection()?;
        redis::cmd("DEL").arg(key).exec(&mut conn)?;
        Ok(())
    }

    pub async fn exists(&self, key: &str) -> Result<bool, redis::RedisError> {
        let mut conn = self.client.get_connection()?;
        redis::cmd("EXISTS").arg(key).query(&mut conn)
    }

    pub async fn expire(&self, key: &str, seconds: usize) -> Result<(), redis::RedisError> {
        let mut conn = self.client.get_connection()?;
        redis::cmd("EXPIRE").arg(key).arg(seconds).exec(&mut conn)?;
        Ok(())
    }

    pub async fn ttl(&self, key: &str) -> Result<i64, redis::RedisError> {
        let mut conn = self.client.get_connection()?;
        redis::cmd("TTL").arg(key).query(&mut conn)
    }

    pub async fn hset(&self, key: &str, field: &str, value: &str) -> Result<(), redis::RedisError> {
        let mut conn = self.client.get_connection()?;
        redis::cmd("HSET")
            .arg(key)
            .arg(field)
            .arg(value)
            .exec(&mut conn)?;
        Ok(())
    }

    pub async fn hget(&self, key: &str, field: &str) -> Result<Option<String>, redis::RedisError> {
        let mut conn = self.client.get_connection()?;
        redis::cmd("HGET").arg(key).arg(field).query(&mut conn)
    }

    pub async fn hdel(&self, key: &str, field: &str) -> Result<(), redis::RedisError> {
        let mut conn = self.client.get_connection()?;
        redis::cmd("HDEL").arg(key).arg(field).exec(&mut conn)?;
        Ok(())
    }

    pub async fn hgetall(
        &self,
        key: &str,
    ) -> Result<std::collections::HashMap<String, String>, redis::RedisError> {
        let mut conn = self.client.get_connection()?;
        redis::cmd("HGETALL").arg(key).query(&mut conn)
    }

    pub async fn lpush(&self, key: &str, value: &str) -> Result<(), redis::RedisError> {
        let mut conn = self.client.get_connection()?;
        redis::cmd("LPUSH").arg(key).arg(value).exec(&mut conn)?;
        Ok(())
    }

    pub async fn rpush(&self, key: &str, value: &str) -> Result<(), redis::RedisError> {
        let mut conn = self.client.get_connection()?;
        redis::cmd("RPUSH").arg(key).arg(value).exec(&mut conn)?;
        Ok(())
    }

    pub async fn lpop(&self, key: &str) -> Result<Option<String>, redis::RedisError> {
        let mut conn = self.client.get_connection()?;
        redis::cmd("LPOP").arg(key).query(&mut conn)
    }

    pub async fn rpop(&self, key: &str) -> Result<Option<String>, redis::RedisError> {
        let mut conn = self.client.get_connection()?;
        redis::cmd("RPOP").arg(key).query(&mut conn)
    }

    pub async fn llen(&self, key: &str) -> Result<i64, redis::RedisError> {
        let mut conn = self.client.get_connection()?;
        redis::cmd("LLEN").arg(key).query(&mut conn)
    }

    pub async fn lrange(
        &self,
        key: &str,
        start: isize,
        stop: isize,
    ) -> Result<Vec<String>, redis::RedisError> {
        let mut conn = self.client.get_connection()?;
        redis::cmd("LRANGE")
            .arg(key)
            .arg(start)
            .arg(stop)
            .query(&mut conn)
    }

    pub async fn sadd(&self, key: &str, member: &str) -> Result<(), redis::RedisError> {
        let mut conn = self.client.get_connection()?;
        redis::cmd("SADD").arg(key).arg(member).exec(&mut conn)?;
        Ok(())
    }

    pub async fn srem(&self, key: &str, member: &str) -> Result<(), redis::RedisError> {
        let mut conn = self.client.get_connection()?;
        redis::cmd("SREM").arg(key).arg(member).exec(&mut conn)?;
        Ok(())
    }

    pub async fn sismember(&self, key: &str, member: &str) -> Result<bool, redis::RedisError> {
        let mut conn = self.client.get_connection()?;
        redis::cmd("SISMEMBER")
            .arg(key)
            .arg(member)
            .query(&mut conn)
    }

    pub async fn smembers(&self, key: &str) -> Result<Vec<String>, redis::RedisError> {
        let mut conn = self.client.get_connection()?;
        redis::cmd("SMEMBERS").arg(key).query(&mut conn)
    }

    pub async fn scard(&self, key: &str) -> Result<i64, redis::RedisError> {
        let mut conn = self.client.get_connection()?;
        redis::cmd("SCARD").arg(key).query(&mut conn)
    }

    pub async fn incr(&self, key: &str) -> Result<i64, redis::RedisError> {
        let mut conn = self.client.get_connection()?;
        redis::cmd("INCR").arg(key).query(&mut conn)
    }

    pub async fn decr(&self, key: &str) -> Result<i64, redis::RedisError> {
        let mut conn = self.client.get_connection()?;
        redis::cmd("DECR").arg(key).query(&mut conn)
    }

    pub async fn incrby(&self, key: &str, increment: i64) -> Result<i64, redis::RedisError> {
        let mut conn = self.client.get_connection()?;
        redis::cmd("INCRBY")
            .arg(key)
            .arg(increment)
            .query(&mut conn)
    }

    pub async fn decrby(&self, key: &str, decrement: i64) -> Result<i64, redis::RedisError> {
        let mut conn = self.client.get_connection()?;
        redis::cmd("DECRBY")
            .arg(key)
            .arg(decrement)
            .query(&mut conn)
    }

    pub async fn flushdb(&self) -> Result<(), redis::RedisError> {
        let mut conn = self.client.get_connection()?;
        redis::cmd("FLUSHDB").exec(&mut conn)?;
        Ok(())
    }

    pub async fn flushall(&self) -> Result<(), redis::RedisError> {
        let mut conn = self.client.get_connection()?;
        redis::cmd("FLUSHALL").exec(&mut conn)?;
        Ok(())
    }

    pub async fn keys(&self, pattern: &str) -> Result<Vec<String>, redis::RedisError> {
        let mut conn = self.client.get_connection()?;
        redis::cmd("KEYS").arg(pattern).query(&mut conn)
    }

    pub async fn ping(&self) -> Result<String, redis::RedisError> {
        let mut conn = self.client.get_connection()?;
        redis::cmd("PING").query(&mut conn)
    }

    pub async fn info(&self) -> Result<String, redis::RedisError> {
        let mut conn = self.client.get_connection()?;
        redis::cmd("INFO").query(&mut conn)
    }

    pub async fn cleanup(&self) -> Result<(), redis::RedisError> {
        self.flushdb().await
    }

    pub async fn seed_test_data(&self) -> Result<TestRedisData, redis::RedisError> {
        // Set up some test data
        self.set("test:key1", "value1").await?;
        self.set("test:key2", "value2").await?;
        self.set("test:key3", "value3").await?;

        // Set up test hash
        self.hset("test:hash", "field1", "hashvalue1").await?;
        self.hset("test:hash", "field2", "hashvalue2").await?;

        // Set up test list
        self.lpush("test:list", "item1").await?;
        self.lpush("test:list", "item2").await?;
        self.lpush("test:list", "item3").await?;

        // Set up test set
        self.sadd("test:set", "member1").await?;
        self.sadd("test:set", "member2").await?;
        self.sadd("test:set", "member3").await?;

        // Set up test data with expiration
        self.set("test:expiring", "temporary").await?;
        self.expire("test:expiring", 60).await?;

        // Set up test counters
        self.set("test:counter", "10").await?;

        Ok(TestRedisData {
            simple_keys: vec!["test:key1", "test:key2", "test:key3"],
            hash_key: "test:hash",
            list_key: "test:list",
            set_key: "test:set",
            expiring_key: "test:expiring",
            counter_key: "test:counter",
        })
    }

    pub async fn clear_test_data(&self) -> Result<(), redis::RedisError> {
        let keys = self.keys("test:*").await?;
        for key in keys {
            self.del(&key).await?;
        }
        Ok(())
    }

    pub async fn verify_connection(&self) -> Result<bool, redis::RedisError> {
        match self.ping().await {
            Ok(response) => Ok(response == "PONG"),
            Err(e) => Err(e),
        }
    }
}

// TestRedis cannot be cloned because Container doesn't support Clone
// Users should create new instances as needed

#[derive(Debug, Clone)]
pub struct TestRedisData {
    pub simple_keys: Vec<&'static str>,
    pub hash_key: &'static str,
    pub list_key: &'static str,
    pub set_key: &'static str,
    pub expiring_key: &'static str,
    pub counter_key: &'static str,
}

pub async fn spawn_test_redis() -> TestRedis {
    TestRedis::new().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_redis_setup() {
        let test_redis = TestRedis::new().await;

        // Test that we can connect to Redis
        let result = test_redis.ping().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "PONG");

        let _ = test_redis.cleanup().await;
    }

    #[tokio::test]
    async fn test_basic_operations() {
        let test_redis = TestRedis::new().await;

        // Test SET and GET
        test_redis.set("test_key", "test_value").await.unwrap();
        let value = test_redis.get("test_key").await.unwrap();
        assert_eq!(value, Some("test_value".to_string()));

        // Test EXISTS
        let exists = test_redis.exists("test_key").await.unwrap();
        assert!(exists);

        // Test DEL
        test_redis.del("test_key").await.unwrap();
        let exists_after_del = test_redis.exists("test_key").await.unwrap();
        assert!(!exists_after_del);

        let _ = test_redis.cleanup().await;
    }

    #[tokio::test]
    async fn test_hash_operations() {
        let test_redis = TestRedis::new().await;

        // Test HSET and HGET
        test_redis
            .hset("test_hash", "field1", "value1")
            .await
            .unwrap();
        let value = test_redis.hget("test_hash", "field1").await.unwrap();
        assert_eq!(value, Some("value1".to_string()));

        // Test HGETALL
        test_redis
            .hset("test_hash", "field2", "value2")
            .await
            .unwrap();
        let all_values = test_redis.hgetall("test_hash").await.unwrap();
        assert_eq!(all_values.len(), 2);
        assert_eq!(all_values.get("field1"), Some(&"value1".to_string()));
        assert_eq!(all_values.get("field2"), Some(&"value2".to_string()));

        let _ = test_redis.cleanup().await;
    }

    #[tokio::test]
    async fn test_list_operations() {
        let test_redis = TestRedis::new().await;

        // Test LPUSH and LLEN
        test_redis.lpush("test_list", "item1").await.unwrap();
        test_redis.lpush("test_list", "item2").await.unwrap();
        let length = test_redis.llen("test_list").await.unwrap();
        assert_eq!(length, 2);

        // Test LRANGE
        let items = test_redis.lrange("test_list", 0, -1).await.unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0], "item2"); // LPUSH adds to the front
        assert_eq!(items[1], "item1");

        // Test LPOP
        let popped = test_redis.lpop("test_list").await.unwrap();
        assert_eq!(popped, Some("item2".to_string()));

        let _ = test_redis.cleanup().await;
    }

    #[tokio::test]
    async fn test_set_operations() {
        let test_redis = TestRedis::new().await;

        // Test SADD and SISMEMBER
        test_redis.sadd("test_set", "member1").await.unwrap();
        test_redis.sadd("test_set", "member2").await.unwrap();

        let is_member = test_redis.sismember("test_set", "member1").await.unwrap();
        assert!(is_member);

        let is_not_member = test_redis.sismember("test_set", "member3").await.unwrap();
        assert!(!is_not_member);

        // Test SCARD
        let cardinality = test_redis.scard("test_set").await.unwrap();
        assert_eq!(cardinality, 2);

        // Test SMEMBERS
        let members = test_redis.smembers("test_set").await.unwrap();
        assert_eq!(members.len(), 2);
        assert!(members.contains(&"member1".to_string()));
        assert!(members.contains(&"member2".to_string()));

        let _ = test_redis.cleanup().await;
    }

    #[tokio::test]
    async fn test_counter_operations() {
        let test_redis = TestRedis::new().await;

        // Test INCR
        let val1 = test_redis.incr("test_counter").await.unwrap();
        assert_eq!(val1, 1);

        let val2 = test_redis.incr("test_counter").await.unwrap();
        assert_eq!(val2, 2);

        // Test INCRBY
        let val3 = test_redis.incrby("test_counter", 5).await.unwrap();
        assert_eq!(val3, 7);

        // Test DECR
        let val4 = test_redis.decr("test_counter").await.unwrap();
        assert_eq!(val4, 6);

        let _ = test_redis.cleanup().await;
    }

    #[tokio::test]
    async fn test_expiration() {
        let test_redis = TestRedis::new().await;

        // Test EXPIRE and TTL
        test_redis.set("test_expiring", "value").await.unwrap();
        test_redis.expire("test_expiring", 10).await.unwrap();

        let ttl = test_redis.ttl("test_expiring").await.unwrap();
        assert!(ttl > 0 && ttl <= 10);

        let _ = test_redis.cleanup().await;
    }

    #[tokio::test]
    async fn test_seed_and_clear_data() {
        let test_redis = TestRedis::new().await;

        // Test seeding data
        let _test_data = test_redis.seed_test_data().await.unwrap();

        // Verify seeded data exists
        let value1 = test_redis.get("test:key1").await.unwrap();
        assert_eq!(value1, Some("value1".to_string()));

        let hash_value = test_redis.hget("test:hash", "field1").await.unwrap();
        assert_eq!(hash_value, Some("hashvalue1".to_string()));

        let list_length = test_redis.llen("test:list").await.unwrap();
        assert_eq!(list_length, 3);

        let set_cardinality = test_redis.scard("test:set").await.unwrap();
        assert_eq!(set_cardinality, 3);

        let counter_value = test_redis.get("test:counter").await.unwrap();
        assert_eq!(counter_value, Some("10".to_string()));

        // Test clearing data
        test_redis.clear_test_data().await.unwrap();

        let value_after_clear = test_redis.get("test:key1").await.unwrap();
        assert_eq!(value_after_clear, None);

        let _ = test_redis.cleanup().await;
    }

    #[tokio::test]
    async fn test_connection_verification() {
        let test_redis = TestRedis::new().await;

        let is_connected = test_redis.verify_connection().await.unwrap();
        assert!(is_connected);

        let _ = test_redis.cleanup().await;
    }

    #[tokio::test]
    async fn test_keys_pattern_matching() {
        let test_redis = TestRedis::new().await;

        // Set up some keys with patterns
        test_redis.set("user:1:name", "John").await.unwrap();
        test_redis.set("user:2:name", "Jane").await.unwrap();
        test_redis
            .set("user:1:email", "john@example.com")
            .await
            .unwrap();
        test_redis.set("session:abc123", "data").await.unwrap();

        // Test pattern matching
        let user_keys = test_redis.keys("user:*").await.unwrap();
        assert_eq!(user_keys.len(), 3);

        let name_keys = test_redis.keys("user:*:name").await.unwrap();
        assert_eq!(name_keys.len(), 2);

        let session_keys = test_redis.keys("session:*").await.unwrap();
        assert_eq!(session_keys.len(), 1);

        let _ = test_redis.cleanup().await;
    }
}
