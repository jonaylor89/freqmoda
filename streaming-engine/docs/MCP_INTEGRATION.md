# MCP Integration for Streaming Engine

This document explains how to integrate the streaming engine audio processing server with LLMs using the Model Context Protocol (MCP).

## What's Included

### 1. Node.js MCP Server (`mcp-server/`)
A lightweight Node.js server that exposes streaming engine functionality to MCP-compatible LLMs like Claude.

**Available Tools:**
- `process_audio` - Process audio files with effects and transformations
- `preview_audio_params` - Preview processing parameters without actually processing
- `get_server_health` - Check if the streaming engine server is running

### 2. OpenAPI Documentation
Your Rust server now exposes OpenAPI schema at:
- `/openapi.json` - Full OpenAPI specification  
- `/api-schema` - Simplified schema for LLM consumption

## Setup Instructions

### 1. Install Dependencies
```bash
./setup-mcp.sh
```

### 2. Start the Streaming Engine Server
```bash
cargo run
```

### 3. Test the MCP Server
```bash
cd mcp-server
node index.js
```

## Integration Options

### Option 1: Claude Desktop (Recommended)

Add this to your Claude Desktop configuration file:

**macOS:** `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows:** `%APPDATA%/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "streaming-audio": {
      "command": "node",
      "args": ["/path/to/streaming-engine/mcp-server/index.js"],
      "env": {
        "STREAMING_ENGINE_SERVER_URL": "http://localhost:8080"
      }
    }
  }
}
```

### Option 2: Direct HTTP API
LLMs that support HTTP tools can use the OpenAPI schema:

```bash
curl http://localhost:8080/api-schema
```

### Option 3: Other MCP Clients
Any MCP-compatible client can connect to the Node.js server using stdio transport.

## Usage Examples

Once connected, you can ask Claude things like:

- "Process this audio file with a reverb effect: https://example.com/audio.mp3"
- "Take this audio and slow it down to half speed with a fade in"
- "Add bass boost and normalize the levels of this track"
- "Preview the parameters for reversing this audio file"

## Environment Variables

- `STREAMING_ENGINE_SERVER_URL` - URL of your streaming engine server (default: http://localhost:8080)

## Supported Audio Effects

All effects from the original streaming engine API are supported:

**Format & Encoding:**
- format, codec, sample_rate, channels, bit_rate, quality

**Time Operations:**
- start_time, duration, speed, reverse

**Volume Operations:**
- volume, normalize, normalize_level

**Audio Effects:**
- lowpass, highpass, bass, treble (numeric values)
- echo, chorus, flanger (use "light", "medium", or "heavy")

**Fades:**
- fade_in, fade_out, cross_fade

**Note:** Custom FFmpeg filters are disabled in the MCP interface to prevent LLMs from generating invalid filter syntax. Use the built-in effects with simple presets instead.

## Troubleshooting

1. **Server not responding:** Make sure streaming engine is running on port 8080
2. **MCP connection issues:** Check that Node.js dependencies are installed
3. **Audio processing errors:** Verify the audio URL is accessible and in a supported format

## Security Notes

- The MCP server uses the `/unsafe/` endpoint - consider implementing proper URL signing for production
- Only use trusted audio sources when processing remote URLs
- The server respects the same rate limiting and security measures as the main streaming engine server