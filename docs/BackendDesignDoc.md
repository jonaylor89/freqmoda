# AudioStreamServer Integration - Backend Design Document

## Overview

This document outlines the architecture and design decisions for integrating AudioStreamServer (formerly Cyberpunk) with Claude AI to create a unified audio processing chat application. The goal is to enable users to request audio manipulations through natural language, which are then processed by the AudioStreamServer.

## Architecture Components

### 1. Gateway Service

A Rust-based gateway service that:
- Provides a unified API endpoint for client applications
- Orchestrates communication between Claude AI and the AudioStreamServer
- Manages conversation state and history
- Processes LLM tool calls into AudioStreamServer requests

### 2. Integration Points

```
+-----------------+     +---------------+     +------------------+
|                 |     |               |     |                  |
|  Client         +---->+  Gateway      +---->+  Claude AI       |
|  Application    |     |  Service      |     |  (Anthropic API) |
|                 |     |               |     |                  |
+-----------------+     +-------+-------+     +------------------+
                                |
                                v
                        +-------+-------+
                        |               |
                        |  AudioStream  |
                        |  Server       |
                        |               |
                        +---------------+
```

## Technical Stack

- **Framework**: Axum (as per your preference)
- **External APIs**:
  - Anthropic Claude API
  - AudioStreamServer API
- **State Management**: PostgreSQL database from the beginning (both prototype and production)
- **Storage**: Google Cloud Storage (GCS) for audio files
- **Serialization**: Serde for JSON handling
- **Error Handling**: Axum's error handling with proper status codes

## Key Endpoints

### 1. `/api/chat` (POST)

Main endpoint for chat functionality:
- Accepts user messages
- Manages conversation context
- Communicates with Claude API
- Processes tool calls for audio manipulation
- Returns responses with processed audio links

**Request Format:**
```json
{
  "message": "String containing user message",
  "conversation_id": "Optional ID for continuing conversations"
}
```


**Response Format:**
```json
{
  "message": "Response from Claude including processed audio links",
  "conversation_id": "ID for this conversation"
}
```

### 2. `/api/audio/process` (POST)

Direct endpoint for audio processing without chat:
- Takes audio processing parameters directly
- Calls AudioStreamServer
- Returns processed audio URL

**Request Format:**
```json
{
  "audio_name": "URL/URI/filename to audio file",
  "parameters": {
    "speed": 1.5,
    "reverse": true,
    "fade_in": 2.0,
    "fade_out": 1.0,
    "echo": "medium"
  }
}
```

### 3. `/health` (GET)

Basic health check endpoint for monitoring.

## Claude API Integration

### Tool Definition

The gateway defines a tool for
 Claude to use for audio processing:

```json
{
  "name": "process_audio",
  "description": "Process audio with various effects and transformations",
  "input_schema": {
    "type": "object",
    "properties": {
      "audio_name": {
        "type": "string",
        "description": "URL/URI/filename to audio file",
      },
      "speed": {
        "type": "number",
        "description": "Playback speed multiplier"
      },
      "reverse": {
        "type": "boolean",
        "description": "Reverse the audio"
      },
      "fade_in": {
        "type": "number",
        "description": "Fade in duration in seconds"
      },
      "fade_out": {
        "type": "number",
        "description": "Fade out duration in seconds"
      },
      "echo": {
        "type": "string",
        "description": "Echo effect - use simple values like 'light', 'medium', or 'heavy'"
      }
    },
    "required": ["audio_name"]
  }
}
```

### Effect Presets

To simplify interaction, the gateway maps simple effect descriptions to actual parameters:

```rust
// Examples of effect mappings
let echo_presets = HashMap::from([
    ("light", "0.6:0.3:1000:0.3"),
    ("medium", "0.8:0.88:60:0.4"),
    ("heavy", "0.8:0.9:1000:0.5"),
]);
```

## AudioStreamServer Integration

The gateway translates Claude's tool calls into proper AudioStreamServer URLs:

1. Extract parameters from tool call
2. Build query string with appropriate parameters
3. Construct URL in format: `{server_url}/unsafe/{audio_name}?{params}`
4. Return URL for client to use (or process audio directly if needed)

## State Management

### Conversation History

Use PostgreSQL database from the beginning for both prototype and production:
- User identification
- Conversation persistence
- Pagination for longer conversations

#### Database Schema (Simplified)

```sql
-- Users table
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(255) NOT NULL UNIQUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Conversations table
CREATE TABLE conversations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id INTEGER REFERENCES users(id),
    title VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Messages table
CREATE TABLE messages (
    id SERIAL PRIMARY KEY,
    conversation_id UUID REFERENCES conversations(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL, -- 'user' or 'assistant'
    content TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Sample audio files (pre-loaded library)
CREATE TABLE audio_samples (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    streaming_key VARCHAR(255) NOT NULL UNIQUE,
    title VARCHAR(255) NOT NULL,
    duration FLOAT,
    file_type VARCHAR(50) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

```

#### Audio Sample Library

The system provides a public library of 10 high-quality audio samples stored on Google Cloud Storage:

The system will be seeded with 10 high-quality audio samples stored on Google Cloud Storage:

-- Sample seed data
INSERT INTO audio_samples (streaming_key, title, duration, file_type) VALUES
('sample1.mp3', 'Sample 1', 8.0,  'audio/mpeg'),
('sample2.mp3', 'Sample 2', 12.5, 'audio/mpeg'),
('sample3.mp3', 'Sample 3', 15.2, 'audio/mpeg'),
('sample4.mp3', 'Sample 4', 10.0,  'audio/mpeg'),
('sample5.mp3', 'Sample 5', 20.0, 'audio/mpeg'),
('sample6.mp3', 'Sample 6', 30.0, 'audio/mpeg'),
('sample7.mp3', 'Sample 7', 5.8,  'audio/mpeg'),
('sample8.mp3', 'Sample 8', 25.5,'audio/mpeg'),
('sample9.mp3', 'Sample 9', 18.0, 'audio/mpeg'),
('sample10.mp3', 'Sample 10', 15.0, 'audio/mpeg');

These samples serve as a public library for all users to experiment with audio processing. All users will have access to the same set of samples through the web interface. The system does not currently support user uploads or saving modified versions.

#### Database Access

Use `sqlx` for type-safe SQL queries:

```rust
// Example: Storing a new message
pub async fn store_message(
    pool: &PgPool,
    conversation_id: &Uuid,
    role: &str,
    content: &str
) -> Result<i32, sqlx::Error> {
    let rec = sqlx::query_as!(
        MessageRecord,
        r#"
        INSERT INTO messages (conversation_id, role, content)
        VALUES ($1, $2, $3)
        RETURNING id, conversation_id, role, content, created_at
        "#,
        conversation_id,
        role,
        content
    )
    .fetch_one(pool)
    .await?;

    Ok(rec.id)
}
```

## Error Handling

Implement comprehensive error handling for:
1. Invalid user inputs
2. Claude API failures
3. AudioStreamServer connectivity issues
4. Malformed audio URLs

Use proper HTTP status codes and return informative error messages.

## Implementation Recommendations

1. **Use Axum Extractors** for request validation
2. **Implement middleware** for common functionality:
   - Authentication (when needed)
   - Logging
   - Error handling
3. **Use Tower layers** for cross-cutting concerns
4. **Separate business logic** from request handlers
5. **Create service abstractions** for Claude and AudioStreamServer

## Setup Instructions

1. **Config**:

Streaming Engine uses a flexible configuration system combining YAML configuration files and environment variables.

Configuration Files

The configuration files are structured as follows:
- `config/base.yml` - Base configuration applied to all environments
- `config/local.yml` - Development environment configuration
- `config/production.yml` - Production environment configuration

You can select which environment to use by setting the `APP_ENVIRONMENT` environment variable to either `local` or `production`.


2. **Integration with Monorepo**:
   - Place gateway in appropriate directory
   - Update workspace configuration if needed
   - Share common libraries for auth, logging, etc.

3. **Dependencies**:
   - Axum for web framework
   - Tokio for async runtime
   - Serde for serialization
   - Reqwest for HTTP clients
   - Anthropic SDK or custom client for Claude API
   - SQLx for PostgreSQL database access with compile-time checked queries
   - uuid for generating unique identifiers
   - tower-http for middleware components
   - google-cloud-storage for GCS integration
   - cloud-storage for Rust GCS client
   - ffmpeg for audio metadata extraction

## Future Enhancements

1. **Authentication and user management** - Expanded role-based permissions
2. **File upload capabilities** - Allow users to upload their own audio files
3. **Credit system for tracking usage** - Paid tiers with different quotas
4. **Streaming audio responses** - Real-time processing feedback
5. **Advanced audio preview functionality** - Waveform visualization
6. **Collaborative editing** - Shared workspaces for teams
7. **Audio library management** - Tagging, categorization, and search

## Example Flow

1. User sends: "Reverse Sample 1 and add echo"
2. Gateway saves the message to PostgreSQL and forwards to Claude with tool definitions
3. Claude identifies intent and calls `process_audio` tool
4. Gateway looks up the sample in the `audio_samples` table by name
5. Gateway translates the tool call to AudioStreamServer URL:
   ```
   http://audiostream-server:8080/unsafe/sample1.mp3?reverse=true&echo=0.8:0.88:60:0.4
   ```
6. Gateway stores the processed audio information in the database
7. Gateway returns Claude's response with the processed audio URL
8. Client displays response and audio player with the URL
9. Conversation history is persisted for future sessions

## Security Considerations

1. **Input validation** for all user-provided URLs and parameters
2. **API key security** - use environment variables or secure vaults
3. **Rate limiting** to prevent abuse
4. **Sanitize error messages** to avoid leaking implementation details
5. **CORS configuration** for web clients
