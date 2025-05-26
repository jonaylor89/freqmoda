-- Create users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    username VARCHAR(255) NOT NULL UNIQUE,
    created_at TIMESTAMP
    WITH
        TIME ZONE DEFAULT NOW ()
);

-- Create conversations table
CREATE TABLE conversations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    user_id UUID REFERENCES users (id),
    title VARCHAR(255),
    created_at TIMESTAMP
    WITH
        TIME ZONE DEFAULT NOW (),
        updated_at TIMESTAMP
    WITH
        TIME ZONE DEFAULT NOW ()
);

-- Create messages table
CREATE TABLE messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    conversation_id UUID REFERENCES conversations (id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL CHECK (role IN ('user', 'assistant')),
    content TEXT NOT NULL,
    created_at TIMESTAMP
    WITH
        TIME ZONE DEFAULT NOW ()
);

-- Create audio_samples table
CREATE TABLE audio_samples (
    streaming_key VARCHAR(255) PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    duration FLOAT,
    file_type VARCHAR(50) NOT NULL,
    created_at TIMESTAMP
    WITH
        TIME ZONE DEFAULT NOW ()
);

-- Create indexes for better performance
CREATE INDEX idx_conversations_user_id ON conversations (user_id);

CREATE INDEX idx_conversations_updated_at ON conversations (updated_at DESC);

CREATE INDEX idx_messages_conversation_id ON messages (conversation_id);

CREATE INDEX idx_messages_created_at ON messages (created_at);

CREATE INDEX idx_audio_samples_streaming_key ON audio_samples (streaming_key);

CREATE INDEX idx_audio_samples_title ON audio_samples (LOWER(title));

-- Insert sample audio files
INSERT INTO
    audio_samples (streaming_key, title, duration, file_type)
VALUES
    ('sample1.mp3', 'Sample 1', 32.86, 'audio/mpeg'),
    ('sample2.mp3', 'Sample 2', 32.86, 'audio/mpeg'),
    ('sample3.mp3', 'Sample 3', 32.86, 'audio/mpeg'),
    ('sample4.mp3', 'Sample 4', 32.86, 'audio/mpeg'),
    ('sample5.mp3', 'Sample 5', 130.63, 'audio/mpeg'),
    ('sample6.mp3', 'Sample 6', 32.86, 'audio/mpeg'),
    ('sample7.mp3', 'Sample 7', 32.86, 'audio/mpeg'),
    ('sample8.mp3', 'Sample 8', 32.86, 'audio/mpeg');
