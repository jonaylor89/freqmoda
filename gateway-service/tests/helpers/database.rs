use gateway_service::config::DatabaseSettings;
use gateway_service::database::get_connection_pool;
use once_cell::sync::Lazy;
use secrecy::SecretString;
use sqlx::{PgPool, Row};
use testcontainers::{Container, clients::Cli};
use testcontainers_modules::postgres::Postgres;
use uuid::Uuid;

static DOCKER: Lazy<Cli> = Lazy::new(Cli::default);

pub struct TestDatabase {
    pub connection_pool: PgPool,
    pub settings: DatabaseSettings,
    _container: Container<'static, Postgres>,
}

impl TestDatabase {
    pub async fn new() -> Self {
        let container = DOCKER.run(Postgres::default());

        let port = container.get_host_port_ipv4(5432);

        let settings = DatabaseSettings {
            username: "postgres".to_string(),
            password: SecretString::new("postgres".into()),
            port,
            host: "127.0.0.1".to_string(),
            database_name: "postgres".to_string(),
            require_ssl: false,
        };

        // Wait for the database to be ready
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let connection_pool = get_connection_pool(&settings);

        // Run migrations manually for testing
        // In production, migrations would be run separately
        Self::run_test_migrations(&connection_pool).await;

        Self {
            connection_pool,
            settings,
            _container: container,
        }
    }

    async fn run_test_migrations(pool: &PgPool) {
        // Create tables for testing - execute statements separately
        sqlx::query(r#"CREATE EXTENSION IF NOT EXISTS "uuid-ossp""#)
            .execute(pool)
            .await
            .expect("Failed to create uuid extension");

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                email VARCHAR(255) NOT NULL UNIQUE,
                name VARCHAR(255) NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(pool)
        .await
        .expect("Failed to create users table");

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS conversations (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                title VARCHAR(255),
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(pool)
        .await
        .expect("Failed to create conversations table");

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS messages (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
                role VARCHAR(50) NOT NULL CHECK (role IN ('user', 'assistant', 'system')),
                content TEXT NOT NULL,
                metadata JSONB,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(pool)
        .await
        .expect("Failed to create messages table");

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS audio_samples (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                name VARCHAR(255) NOT NULL UNIQUE,
                filename VARCHAR(255) NOT NULL,
                file_path VARCHAR(500) NOT NULL,
                duration DECIMAL(10,2) NOT NULL,
                file_size BIGINT,
                mime_type VARCHAR(100),
                sample_rate INTEGER,
                channels INTEGER,
                bitrate INTEGER,
                metadata JSONB,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(pool)
        .await
        .expect("Failed to create audio_samples table");
    }

    pub async fn cleanup(&self) {
        self.connection_pool.close().await;
    }

    pub async fn seed_test_data(&self) -> TestData {
        let user_id = self.create_test_user("test@example.com").await;
        let conversation_id = self.create_test_conversation(user_id).await;
        let sample_ids = self.create_test_audio_samples().await;

        TestData {
            user_id,
            conversation_id,
            sample_ids,
        }
    }

    async fn create_test_user(&self, email: &str) -> Uuid {
        let user_id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO users (id, email, name, created_at)
            VALUES ($1, $2, $3, NOW())
            "#,
        )
        .bind(user_id)
        .bind(email)
        .bind("Test User")
        .execute(&self.connection_pool)
        .await
        .expect("Failed to create test user");

        user_id
    }

    async fn create_test_conversation(&self, user_id: Uuid) -> Uuid {
        let conversation_id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO conversations (id, user_id, created_at, updated_at)
            VALUES ($1, $2, NOW(), NOW())
            "#,
        )
        .bind(conversation_id)
        .bind(user_id)
        .execute(&self.connection_pool)
        .await
        .expect("Failed to create test conversation");

        conversation_id
    }

    async fn create_test_audio_samples(&self) -> Vec<Uuid> {
        let mut sample_ids = Vec::new();

        for i in 1..=5 {
            let sample_id = Uuid::new_v4();
            let name = format!("Test Sample {}", i);
            let filename = format!("test_sample_{}.mp3", i);

            sqlx::query(
                r#"
                INSERT INTO audio_samples (id, name, filename, file_path, duration, created_at)
                VALUES ($1, $2, $3, $4, $5, NOW())
                "#,
            )
            .bind(sample_id)
            .bind(&name)
            .bind(&filename)
            .bind(format!("/test/samples/{}", filename))
            .bind(30.0 + (i as f64) * 5.0)
            .execute(&self.connection_pool)
            .await
            .expect("Failed to create test audio sample");

            sample_ids.push(sample_id);
        }

        sample_ids
    }

    pub async fn create_test_message(
        &self,
        conversation_id: Uuid,
        role: &str,
        content: &str,
    ) -> Uuid {
        let message_id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO messages (id, conversation_id, role, content, created_at)
            VALUES ($1, $2, $3, $4, NOW())
            "#,
        )
        .bind(message_id)
        .bind(conversation_id)
        .bind(role)
        .bind(content)
        .execute(&self.connection_pool)
        .await
        .expect("Failed to create test message");

        message_id
    }

    pub async fn get_conversation_message_count(&self, conversation_id: Uuid) -> i64 {
        let row = sqlx::query("SELECT COUNT(*) as count FROM messages WHERE conversation_id = $1")
            .bind(conversation_id)
            .fetch_one(&self.connection_pool)
            .await
            .expect("Failed to get message count");

        row.get::<i64, _>("count")
    }

    pub async fn get_audio_sample_by_name(&self, name: &str) -> Option<AudioSampleRecord> {
        let result = sqlx::query(
            r#"
            SELECT id, name, filename, file_path, duration
            FROM audio_samples
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(&self.connection_pool)
        .await
        .expect("Failed to fetch audio sample");

        result.map(|row| AudioSampleRecord {
            id: row.get("id"),
            name: row.get("name"),
            filename: row.get("filename"),
            file_path: row.get("file_path"),
            duration: row.get("duration"),
        })
    }

    pub async fn clear_conversations(&self) {
        sqlx::query("DELETE FROM messages")
            .execute(&self.connection_pool)
            .await
            .expect("Failed to clear messages");

        sqlx::query("DELETE FROM conversations")
            .execute(&self.connection_pool)
            .await
            .expect("Failed to clear conversations");
    }

    pub async fn clear_users(&self) {
        self.clear_conversations().await;

        sqlx::query("DELETE FROM users")
            .execute(&self.connection_pool)
            .await
            .expect("Failed to clear users");
    }

    pub async fn truncate_all_tables(&self) {
        let tables = vec!["messages", "conversations", "users", "audio_samples"];

        for table in tables {
            sqlx::query(&format!(
                "TRUNCATE TABLE {} RESTART IDENTITY CASCADE",
                table
            ))
            .execute(&self.connection_pool)
            .await
            .unwrap_or_else(|_| panic!("Failed to truncate table {}", table));
        }
    }
}

#[derive(Debug, Clone)]
pub struct TestData {
    pub user_id: Uuid,
    pub conversation_id: Uuid,
    pub sample_ids: Vec<Uuid>,
}

#[derive(Debug, Clone)]
pub struct AudioSampleRecord {
    pub id: Uuid,
    pub name: String,
    pub filename: String,
    pub file_path: String,
    pub duration: f64,
}

pub async fn spawn_test_database() -> TestDatabase {
    TestDatabase::new().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_setup() {
        let test_db = TestDatabase::new().await;

        // Test that we can connect to the database
        let result = sqlx::query("SELECT 1 as test")
            .fetch_one(&test_db.connection_pool)
            .await;

        assert!(result.is_ok());
        test_db.cleanup().await;
    }

    #[tokio::test]
    async fn test_seed_test_data() {
        let test_db = TestDatabase::new().await;

        let test_data = test_db.seed_test_data().await;

        assert!(!test_data.user_id.is_nil());
        assert!(!test_data.conversation_id.is_nil());
        assert_eq!(test_data.sample_ids.len(), 5);

        test_db.cleanup().await;
    }

    #[tokio::test]
    async fn test_create_and_retrieve_message() {
        let test_db = TestDatabase::new().await;
        let test_data = test_db.seed_test_data().await;

        let message_content = "Test message content";
        let message_id = test_db
            .create_test_message(test_data.conversation_id, "user", message_content)
            .await;

        assert!(!message_id.is_nil());

        let count = test_db
            .get_conversation_message_count(test_data.conversation_id)
            .await;
        assert_eq!(count, 1);

        test_db.cleanup().await;
    }

    #[tokio::test]
    async fn test_audio_sample_retrieval() {
        let test_db = TestDatabase::new().await;
        let _test_data = test_db.seed_test_data().await;

        let sample = test_db.get_audio_sample_by_name("Test Sample 1").await;

        assert!(sample.is_some());
        let sample = sample.unwrap();
        assert_eq!(sample.name, "Test Sample 1");
        assert_eq!(sample.filename, "test_sample_1.mp3");

        test_db.cleanup().await;
    }

    #[tokio::test]
    async fn test_cleanup_operations() {
        let test_db = TestDatabase::new().await;
        let test_data = test_db.seed_test_data().await;

        // Add some messages
        test_db
            .create_test_message(test_data.conversation_id, "user", "Test message 1")
            .await;
        test_db
            .create_test_message(test_data.conversation_id, "assistant", "Test response 1")
            .await;

        let count_before = test_db
            .get_conversation_message_count(test_data.conversation_id)
            .await;
        assert_eq!(count_before, 2);

        // Clear conversations should also clear messages
        test_db.clear_conversations().await;

        let count_after = test_db
            .get_conversation_message_count(test_data.conversation_id)
            .await;
        assert_eq!(count_after, 0);

        test_db.cleanup().await;
    }
}
