# Web Demo

A Rust-based web demo that orchestrates communication between Claude AI and the Streaming Engine for audio processing through natural language chat.

## Features

- 🤖 **Claude AI Integration**: Natural language processing for audio manipulation requests
- 🎵 **Audio Processing**: Direct integration with the streaming engine
- 💬 **Chat Interface**: Conversation management with persistent history
- 🗄️ **Database**: PostgreSQL for storing conversations and audio sample library
- 🔧 **RESTful API**: Clean HTTP endpoints for client applications

## Quick Start

### Prerequisites

- Rust 1.70+
- PostgreSQL 14+
- Redis 6+
- Running streaming-engine instance
- Claude API key from Anthropic

### Environment Setup

1. **Database Setup**
   ```bash
   # Create database
   createdb freqmoda_dev

   # Set database URL
   export DATABASE_URL="postgresql://localhost:5432/freqmoda_dev"
   ```

2. **API Keys**
   ```bash
   export CLAUDE_API_KEY="your-claude-api-key"
   ```

3. **Redis Setup**
   ```bash
   # Start Redis (macOS with Homebrew)
   brew services start redis
   
   # Or run Redis directly
   redis-server
   ```

### Running the Service

```bash
# Development mode
cargo run

# Production mode
APP_ENVIRONMENT=production cargo run

# With environment variables
WEB_DEMO_DATABASE__URL="postgresql://localhost:5432/freqmoda_dev" \
WEB_DEMO_CLAUDE__API_KEY="your-api-key" \
cargo run
```

The service will start on `http://localhost:9000` by default.

**Access the Web Interface**: Open `http://localhost:9000` in your browser for the chat interface.

## API Endpoints

### Chat
- `POST /api/chat` - Send messages and get AI responses with audio processing

### Audio Processing
- `POST /api/audio/process` - Direct audio processing without chat
- `GET /api/audio/samples` - List all available audio samples
- `POST /api/audio/metadata` - Get metadata for audio files

### Health
- `GET /health` - Service health check

## Web Chat Interface

The web demo includes a built-in web interface for easy access to AI audio processing:

### Quick Access

Simply visit `http://localhost:9000` in your browser after starting the service.

### Features

- 🌐 **Browser-Based**: No installation required, works in any modern browser
- 🛡️ **Rate Limited Demo**: 20 requests per session, 10 requests per minute per IP
- 💾 **Session Memory**: Conversations persist within your browser session
- 🎵 **Audio Playback**: Direct links to processed audio files
- 📱 **Responsive Design**: Works on desktop and mobile devices

### Demo Mode Limitations

The web interface runs in demo mode with built-in rate limiting:
- **20 requests per session** (resets after 24 hours)
- **10 requests per minute per IP**
- **1000 requests per hour globally** (safety limit)
- **Sessions expire after 24 hours**

### Usage

1. Open `http://localhost:9000` in your browser
2. Type natural language audio processing requests
3. Click "Send" or press Enter
4. View AI responses and click audio links to hear results

### Example Requests

Try these sample requests to get started:
- "Reverse Sample 1 and add echo"
- "Make Sample 2 play faster with a fade in"
- "Add chorus effect to Sample 3"

## CLI Chat Tool

The web demo also includes a CLI tool for interactive chatting:

### Building and Running

```bash
# Build the CLI tool
cargo build --bin chat-cli

# Run directly
cargo run --bin chat-cli

# Or use the convenience script
./scripts/chat.sh
```

### Usage

```bash
# Connect to default web UI (localhost:9000)
cargo run --bin chat-cli

# Connect to custom URL
cargo run --bin chat-cli -- --url http://localhost:4000

# Using the script with custom URL
./scripts/chat.sh --url http://localhost:9000
```

### Features

- 🎯 **Interactive REPL**: Real-time chat interface with command history
- 🔄 **Conversation Memory**: Maintains conversation context across messages
- ⚡ **Quick Exit**: Type `exit` or press `Ctrl+C` to quit
- 🎨 **Emoji Indicators**: Visual feedback for different states
- 📝 **Error Handling**: Clear error messages for connection issues

### Example Session

```
🤖 FreqModa Chat CLI
Connected to: http://localhost:9000
Type 'exit' or press Ctrl+C to quit
───────────────────────────────────────
💬 You: Make Sample 1 play faster and add echo
🤖 Assistant: I've processed Sample 1 with increased speed and echo effect.

Processed audio: http://localhost:8080/unsafe/sample1.mp3?speed=1.5&echo=0.8:0.88:60:0.4

💬 You: Now reverse it
🤖 Assistant: I've applied reverse effect to the previously processed audio.

Processed audio: http://localhost:8080/unsafe/sample1.mp3?speed=1.5&echo=0.8:0.88:60:0.4&reverse=true

💬 You: exit
👋 Goodbye!
```

## Configuration

The service uses a layered configuration system:

1. `config/base.yml` - Base configuration
2. `config/{environment}.yml` - Environment-specific config
3. Environment variables with `WEB_DEMO_` prefix

### Example: Chat Request

```json
{
  "message": "Reverse Sample 1 and add a medium echo effect",
  "conversation_id": "optional-uuid-for-continuing-conversation"
}
```

### Example: Chat Response

```json
{
  "message": "I've processed Sample 1 with reverse effect and medium echo.\n\nProcessed audio: http://localhost:8080/unsafe/sample1.mp3?reverse=true&echo=0.8:0.88:60:0.4",
  "conversation_id": "uuid-of-conversation"
}
```

## Development

### Database Migrations

Migrations are automatically run on startup. The initial migration creates:
- Users table
- Conversations table
- Messages table
- Audio samples table (pre-seeded with 10 samples)

### Adding New Audio Effects

1. Update the Claude tool definition in `services/claude.rs`
2. Add effect preset mappings if needed
3. The streaming engine handles the actual audio processing

### Testing

```bash
# Run tests
cargo test

# Run with coverage
cargo tarpaulin
```

## Project Structure

```
web-demo/
├── src/
│   ├── handlers/          # HTTP request handlers
│   ├── services/          # External service integrations
│   ├── models.rs          # Data models and DTOs
│   ├── database.rs        # Database operations
│   ├── config.rs          # Configuration management
│   ├── error.rs           # Error handling
│   ├── routes.rs          # Route definitions
│   └── state.rs           # Application state
├── config/                # Configuration files
├── migrations/            # Database migrations
└── Cargo.toml
```

## Integration with Streaming Engine

The web demo acts as a bridge between natural language requests and the streaming engine's audio processing capabilities. It:

1. Receives natural language requests via chat
2. Uses Claude AI to understand intent and extract parameters
3. Translates Claude's tool calls into streaming engine URLs
4. Returns processed audio URLs to the client

Sample flow:
```
User: "Make Sample 1 play faster and add echo"
Claude: Identifies audio_name="Sample 1", speed=1.5, echo="medium"
Web Demo: Translates to streaming engine URL
Result: http://localhost:8080/unsafe/sample1.mp3?speed=1.5&echo=0.8:0.88:60:0.4
```
