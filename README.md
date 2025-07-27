
# FreqModa

AI-powered audio processing chat application that lets users manipulate audio files through natural language conversations.

## Architecture

FreqModa consists of several integrated services:

- **Streaming Engine**: Core audio processing server (Rust)
- **Gateway Service**: AI chat orchestrator with Claude integration (Rust)
- **MCP Server**: Model Context Protocol integration for direct LLM access (Node.js)

## Quick Start

### Prerequisites

- Rust 1.70+
- Docker
- Claude API key from Anthropic

## Services

### Streaming Engine (`streaming-engine/`)

Advanced audio processing server that handles real-time audio manipulation.

- **Port**: 8080
- **Features**: Format conversion, effects, filters, time manipulation
- **API**: RESTful with streaming support
- **Storage**: Local, S3, GCS support

[📖 Full Documentation](./streaming-engine/README.md)

### Gateway Service (`gateway-service/`)

Rust-based API gateway that orchestrates communication between Claude AI and the streaming engine.

- **Port**: 9000
- **Features**: Natural language processing, conversation management, audio processing orchestration
- **Database**: PostgreSQL for conversation history
- **AI**: Claude 3.5 Sonnet integration

[📖 Full Documentation](./gateway-service/README.md)

### Web Interface

The gateway service includes a built-in web interface available at http://localhost:9000:

- **Features**: Chat interface, audio playback, sample library
- **Demo Mode**: Rate-limited for demonstration purposes
- **Responsive**: Works on desktop and mobile devices

## API Examples

### Chat with AI for Audio Processing

```bash
curl -X POST http://localhost:9000/api/chat \
  -H "Content-Type: application/json" \
  -d '{
    "message": "Reverse Sample 1 and add a medium echo effect"
  }'
```

### Direct Audio Processing

```bash
curl -X POST http://localhost:9000/api/audio/process \
  -H "Content-Type: application/json" \
  -d '{
    "audio_name": "sample1.mp3",
    "parameters": {
      "reverse": true,
      "echo": "medium",
      "fade_in": 1.0
    }
  }'
```

### Streaming Engine Direct Access

```bash
# Process audio with effects
curl "http://localhost:8080/unsafe/sample1.mp3?reverse=true&echo=0.8:0.88:60:0.4&fade_in=1"

# Get audio metadata
curl "http://localhost:8080/meta/unsafe/sample1.mp3"
```

## Audio Sample Library

The system includes a pre-loaded library of 10 high-quality audio samples:

- **Sample 1**
- **Sample 2** 
- **Sample 3**
- **Sample 4** 
- **Sample 5**
- **Sample 6** 
- **Sample 7** 
- **Sample 8** 
- **Sample 9**
- **Sample 10**

## Development

### Project Structure

```
freqmoda/
├── streaming-engine/          # Core audio processing server
│   └── mcp-server/           # MCP integration for direct LLM access
├── gateway-service/           # AI chat orchestrator  
├── mcp-server/               # Shared MCP artifacts
├── scripts/                  # Development utilities
├── docs/                     # Project documentation
└── migrations/               # Database schema migrations
```
