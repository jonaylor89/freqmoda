-- Migration: Create audio_versions table
-- This table tracks different processed versions of audio samples

CREATE TABLE audio_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sample_id VARCHAR NOT NULL, -- Links to AudioSample.streaming_key
    session_id VARCHAR NOT NULL, -- Links to user session
    conversation_id UUID, -- Links to conversation if applicable
    audio_url VARCHAR NOT NULL, -- URL to the processed audio file
    description TEXT, -- Description of what was done (e.g., "Added echo effect")
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE SET NULL
);

-- Index for faster lookups
CREATE INDEX idx_audio_versions_sample_session 
    ON audio_versions(sample_id, session_id, created_at DESC);

CREATE INDEX idx_audio_versions_conversation 
    ON audio_versions(conversation_id);

-- Note: Original versions will be inserted by the application code
-- since we need the streaming_engine_base_url from config
