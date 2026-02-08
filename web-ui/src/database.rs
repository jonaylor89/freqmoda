use crate::config::DatabaseSettings;
use crate::error::Result;
use crate::models::{AudioSample, Conversation, Message, User};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .max_connections(10)
        .min_connections(2)
        .acquire_timeout(std::time::Duration::from_secs(10))
        .idle_timeout(std::time::Duration::from_secs(600))
        .max_lifetime(std::time::Duration::from_secs(1800))
        .connect_lazy_with(configuration.with_db())
}

// User operations
pub async fn create_user_if_not_exists(pool: &PgPool, username: &str) -> Result<User> {
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (username) VALUES ($1)
         ON CONFLICT (username) DO UPDATE SET username = $1
         RETURNING id, username, created_at",
    )
    .bind(username)
    .fetch_one(pool)
    .await?;

    Ok(user)
}

pub async fn get_user_by_username(pool: &PgPool, username: &str) -> Result<Option<User>> {
    let user =
        sqlx::query_as::<_, User>("SELECT id, username, created_at FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(pool)
            .await?;

    Ok(user)
}

// Conversation operations
pub async fn create_conversation(
    pool: &PgPool,
    user_id: Option<Uuid>,
    title: Option<String>,
) -> Result<Conversation> {
    let conversation = sqlx::query_as::<_, Conversation>(
        "INSERT INTO conversations (user_id, title) VALUES ($1, $2)
         RETURNING id, user_id, title, created_at, updated_at",
    )
    .bind(user_id)
    .bind(title)
    .fetch_one(pool)
    .await?;

    Ok(conversation)
}

pub async fn get_conversation(
    pool: &PgPool,
    conversation_id: &Uuid,
) -> Result<Option<Conversation>> {
    let conversation = sqlx::query_as::<_, Conversation>(
        "SELECT id, user_id, title, created_at, updated_at
         FROM conversations WHERE id = $1",
    )
    .bind(conversation_id)
    .fetch_optional(pool)
    .await?;

    Ok(conversation)
}

pub async fn update_conversation_timestamp(pool: &PgPool, conversation_id: &Uuid) -> Result<()> {
    sqlx::query("UPDATE conversations SET updated_at = NOW() WHERE id = $1")
        .bind(conversation_id)
        .execute(pool)
        .await?;

    Ok(())
}

// Message operations
pub async fn store_message(
    pool: &PgPool,
    conversation_id: &Uuid,
    role: &str,
    content: &str,
) -> Result<Message> {
    let message = sqlx::query_as::<_, Message>(
        "INSERT INTO messages (conversation_id, role, content)
         VALUES ($1, $2, $3)
         RETURNING id, conversation_id, role, content, created_at",
    )
    .bind(conversation_id)
    .bind(role)
    .bind(content)
    .fetch_one(pool)
    .await?;

    Ok(message)
}

pub async fn get_conversation_messages(
    pool: &PgPool,
    conversation_id: &Uuid,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<Message>> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);

    let messages = sqlx::query_as::<_, Message>(
        "SELECT id, conversation_id, role, content, created_at
         FROM messages
         WHERE conversation_id = $1
         ORDER BY created_at ASC
         LIMIT $2 OFFSET $3",
    )
    .bind(conversation_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(messages)
}

// Audio sample operations
pub async fn get_all_audio_samples(pool: &PgPool) -> Result<Vec<AudioSample>> {
    let samples = sqlx::query_as::<_, AudioSample>(
        "SELECT streaming_key, title, duration, file_type, created_at
         FROM audio_samples
         ORDER BY title",
    )
    .fetch_all(pool)
    .await?;

    Ok(samples)
}

pub async fn get_audio_sample_by_key(pool: &PgPool, key: &str) -> Result<Option<AudioSample>> {
    let sample = sqlx::query_as::<_, AudioSample>(
        "SELECT streaming_key, title, duration, file_type, created_at
         FROM audio_samples
         WHERE streaming_key = $1",
    )
    .bind(key)
    .fetch_optional(pool)
    .await?;

    Ok(sample)
}

pub async fn get_audio_sample_by_title(pool: &PgPool, title: &str) -> Result<Option<AudioSample>> {
    let sample = sqlx::query_as::<_, AudioSample>(
        "SELECT streaming_key, title, duration, file_type, created_at
         FROM audio_samples
         WHERE LOWER(title) = LOWER($1)",
    )
    .bind(title)
    .fetch_optional(pool)
    .await?;

    Ok(sample)
}

pub async fn seed_audio_samples(pool: &PgPool) -> Result<()> {
    let samples = vec![
        ("sample1.mp3", "Sample 1", Some(8.0), "audio/mpeg"),
        ("sample2.mp3", "Sample 2", Some(12.5), "audio/mpeg"),
        ("sample3.mp3", "Sample 3", Some(15.2), "audio/mpeg"),
        ("sample4.mp3", "Sample 4", Some(10.0), "audio/mpeg"),
        ("sample5.mp3", "Sample 5", Some(20.0), "audio/mpeg"),
        ("sample6.mp3", "Sample 6", Some(30.0), "audio/mpeg"),
        ("sample7.mp3", "Sample 7", Some(5.8), "audio/mpeg"),
        ("sample8.mp3", "Sample 8", Some(25.5), "audio/mpeg"),
        ("sample9.mp3", "Sample 9", Some(18.0), "audio/mpeg"),
        ("sample10.mp3", "Sample 10", Some(15.0), "audio/mpeg"),
    ];

    for (streaming_key, title, duration, file_type) in samples {
        sqlx::query(
            "INSERT INTO audio_samples (streaming_key, title, duration, file_type)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (streaming_key) DO NOTHING",
        )
        .bind(streaming_key)
        .bind(title)
        .bind(duration)
        .bind(file_type)
        .execute(pool)
        .await?;
    }

    Ok(())
}
