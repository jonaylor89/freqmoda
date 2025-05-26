# Streaming Engine MCP Server

MCP (Model Context Protocol) server for [streaming-engine](https://github.com/jonaylor89/freqmoda/tree/main/streaming-engine) - Connect LLMs like Claude to your audio processing server.

## Installation

Clone the repository and set up the MCP server:

```bash
git clone https://github.com/jonaylor89/freqmoda.git
cd freqmoda/streaming-engine/mcp-server
npm install
```

## Claude Desktop Integration

Add this to your Claude Desktop config:

**macOS:** `~/Library/Application Support/Claude/claude_desktop_config.json`  
**Windows:** `%APPDATA%/Claude/claude_desktop_config.json`

### For a local streaming engine server:
```json
{
  "mcpServers": {
    "streaming-engine-audio": {
      "command": "node",
      "args": [
        "/path/to/freqmoda/streaming-engine/mcp-server/cli.js"
      ],
      "env": {
        "STREAMING_ENGINE_SERVER_URL": "http://localhost:8080"
      }
    }
  }
}
```

### For a deployed server:
```json
{
  "mcpServers": {
    "streaming-engine-audio": {
      "command": "node",
      "args": [
        "/path/to/freqmoda/streaming-engine/mcp-server/cli.js"
      ],
      "env": {
        "STREAMING_ENGINE_SERVER_URL": "https://your-streaming-engine-server.run.app"
      }
    }
  }
}
```

**Note:** Replace `/path/to/freqmoda` with the actual path where you cloned the repository.

## Available Tools

### `process_audio`
Process audio files with effects and transformations:
- **Time operations**: start_time, duration, speed, reverse
- **Volume**: volume, normalize
- **Filters**: lowpass, highpass, bass, treble  
- **Effects**: echo, chorus, flanger (use "light", "medium", "heavy")
- **Fades**: fade_in, fade_out

### `preview_audio_params`
Preview processing parameters without actually processing the audio

### `get_server_health`
Check if your streaming engine server is running

## Usage Examples

Ask Claude:
- "Process this audio with a medium echo: https://example.com/song.mp3"
- "Slow down this track to half speed and add a fade in"
- "Add heavy bass boost and normalize the levels"
- "Take the first 30 seconds and reverse it"

## Requirements

- Node.js 18+
- Running streaming engine server (local or deployed)
- MCP-compatible LLM (Claude Desktop, etc.)

## Testing the Setup

To test that everything is working:

1. **Start your streaming engine server** (if running locally):
   ```bash
   cd freqmoda/streaming-engine
   cargo run
   ```

2. **Test the MCP server directly**:
   ```bash
   cd freqmoda/streaming-engine/mcp-server
   STREAMING_ENGINE_SERVER_URL=http://localhost:8080 node cli.js
   ```

3. **Restart Claude Desktop** after updating the config file

## Development

```bash
git clone https://github.com/jonaylor89/freqmoda.git
cd freqmoda/streaming-engine/mcp-server
npm install
npm start
```

## License

MIT