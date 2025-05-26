# Streaming Engine

Advanced Audio Processing Server for on the fly audio manipulation

## Quick Start

### Local Development
```sh
# Default build with filesystem storage
cargo run

# Or explicitly specify filesystem feature
cargo run --features filesystem
```

### MCP Integration (Connect to LLMs)
```sh
# Start server, then in another terminal:
npx @streaming-engine/mcp-server

# Or connect to deployed server:
npx @streaming-engine/mcp-server --server=https://your-app.run.app
```
See [MCP_INTEGRATION.md](MCP_INTEGRATION.md) for Claude Desktop setup.

Original audio test file:
```
https://raw.githubusercontent.com/jonaylor89/freqmoda/main/streaming-engine/testdata/celtic_pt2.mp3
```

Try out the following audio manipulation requests:
```
http://localhost:8080/unsafe/https://raw.githubusercontent.com/jonaylor89/freqmoda/main/streaming-engine/main/testdata/celtic_pt2.mp3
http://localhost:8080/unsafe/https://raw.githubusercontent.com/jonaylor89/freqmoda/main/streaming-engine/main/testdata/celtic_pt2.mp3?reverse=true
http://localhost:8080/unsafe/https://raw.githubusercontent.com/jonaylor89/freqmoda/main/streaming-engine/main/testdata/celtic_pt2.mp3?start_time=0&duration=10
http://localhost:8080/unsafe/https://raw.githubusercontent.com/jonaylor89/freqmoda/main/streaming-engine/main/testdata/celtic_pt2.mp3?reverse=true&fade_in=1&fade_out=1&speed=0.8
```

## Streaming Engine API

The Streaming Engine endpoint follows this URL structure:

```
/HASH|unsafe/AUDIO?param1=value1&param2=value2&...
```

- `HASH` is the URL signature hash, or `unsafe` if unsafe mode is used
- `AUDIO` is the audio URI (local file or remote URL)

### Supported Parameters

Streaming Engine supports a wide range of audio processing capabilities:

#### Format & Encoding
- `format` - Output format (mp3, wav, etc.)
- `codec` - Audio codec
- `sample_rate` - Sample rate in Hz
- `channels` - Number of audio channels
- `bit_rate` - Bit rate in kbps
- `bit_depth` - Bit depth
- `quality` - Encoding quality (0.0-1.0)
- `compression_level` - Compression level

#### Time Operations
- `start_time` - Start time in seconds
- `duration` - Duration in seconds
- `speed` - Playback speed multiplier
- `reverse` - Reverse audio (true/false)

#### Volume Operations
- `volume` - Volume adjustment multiplier
- `normalize` - Normalize audio levels (true/false)
- `normalize_level` - Target normalization level in dB

#### Audio Effects
- `lowpass` - Lowpass filter cutoff frequency
- `highpass` - Highpass filter cutoff frequency
- `bandpass` - Bandpass filter parameters
- `bass` - Bass boost/cut level
- `treble` - Treble boost/cut level
- `echo` - Echo effect parameters
- `chorus` - Chorus effect parameters
- `flanger` - Flanger effect parameters
- `phaser` - Phaser effect parameters
- `tremolo` - Tremolo effect parameters
- `compressor` - Compressor effect parameters
- `noise_reduction` - Noise reduction parameters

#### Fades
- `fade_in` - Fade in duration in seconds
- `fade_out` - Fade out duration in seconds
- `cross_fade` - Cross-fade duration in seconds

#### Advanced
- `custom_filters` - Custom FFmpeg filter parameters
- `custom_options` - Custom FFmpeg options
- `tags` - Metadata tags (as `tag_NAME=VALUE`)

### Preview Parameters with `/params`

You can preview the parameters for any request by adding `/params` before the endpoint:

```sh
curl "http://localhost:8080/params/unsafe/celtic_pt2.mp3?reverse=true&fade_in=1"

{
  "audio": "celtic_pt2.mp3",
  "reverse": true,
  "fade_in": 1.0
}
```

### Audio Metadata with `/meta`

You can extract audio metadata by adding `/meta` before the endpoint:

```sh
curl "http://localhost:8080/meta/unsafe/celtic_pt2.mp3"

{
  "format": "mp3",
  "duration": 27.4808,
  "bit_rate": 161192,
  "sample_rate": 44100,
  "channels": 2,
  "codec": "mp3",
  "size": 553712,
  "tags": {
    "iTunSMPB": "...",
    "TSS": "Logic Pro X 10.3.3",
    "iTunNORM": "..."
  }
}
```

## Storage Options

Streaming Engine supports multiple storage backends via cargo features:

- [x] **Local File System** (`filesystem` feature - default)
- [x] **AWS S3 / MinIO** (`s3` feature)
- [x] **Google Cloud Storage (GCS)** (`gcs` feature)
- [_] Decentralized Storage (IPFS, Audius)

### Building with Different Storage Backends

```sh
# Local filesystem (default)
cargo build --features filesystem

# Google Cloud Storage
cargo build --features gcs

# AWS S3/MinIO
cargo build --features s3

# Multiple backends (if needed)
cargo build --features "filesystem,gcs"
```

### Build Script

For convenience, use the included build script:

```sh
# Build with filesystem backend (default)
./build.sh

# Build with specific backend
./build.sh gcs
./build.sh s3

# Build release version
./build.sh gcs release
./build.sh filesystem prod
```

### Production Builds

The Docker build uses GCS by default for production:
```dockerfile
RUN cargo build --release --features gcs
```

This approach significantly reduces build times by only compiling the storage backend you need.

## Advanced Features

- [x] Audio format conversion
- [x] Audio time manipulation (slicing, speed, reverse)
- [x] Audio effects and filters
- [x] Audio fades
- [x] Request caching
- [x] Storage abstraction
- [x] Metrics and monitoring
- [x] Remote audio fetching


## Configuration

Streaming Engine uses a flexible configuration system combining YAML configuration files and environment variables.

### Configuration Files

The configuration files are structured as follows:
- `config/base.yml` - Base configuration applied to all environments
- `config/local.yml` - Development environment configuration
- `config/production.yml` - Production environment configuration

You can select which environment to use by setting the `APP_ENVIRONMENT` environment variable to either `local` or `production`.

### Configuration Structure

#### Application Settings
```yaml
application:
  port: 8080                         # Port the server listens on
  host: "127.0.0.1"                  # Host the server binds to
  hmac_secret: "your-secret-key"     # Secret for URL signing
```

#### Storage Settings
```yaml
storage:
  base_dir: "/path/to/storage"       # Base directory for file storage
  path_prefix: "audio/"              # Prefix for storage paths
  safe_chars: "default"              # Character sanitization mode
  client:                            # Optional storage client configuration
    # S3 Storage Configuration
    S3:
      region: "us-east-1"
      bucket: "my-bucket"
      endpoint: "https://s3.amazonaws.com"  # S3-compatible endpoint URL
      access_key: "access-key"
      secret_key: "secret-key"

    # Or Google Cloud Storage Configuration
    GCS:
      bucket: "my-gcs-bucket"
```

#### Processor Settings
```yaml
processor:
  disabled_filters: []               # List of disabled audio filters
  max_filter_ops: 256               # Maximum number of filter operations
  concurrency: 4                    # Concurrent processing threads (null = auto)
  max_cache_files: 1000             # Maximum number of cached files
  max_cache_mem: 256                # Maximum memory used for caching (MB)
  max_cache_size: 1024              # Maximum cache size (MB)
```

#### Cache Settings
```yaml
cache:
  # Redis Cache Configuration
  Redis:
    uri: "redis://localhost:6379"

  # Or Filesystem Cache Configuration
  Filesystem:
    base_dir: "/path/to/cache"      # Cache directory
```

#### Custom Tags
```yaml
custom_tags:
  env: "production"                 # Custom metadata tags
  version: "1.0.0"
```

### Environment Variables

All configuration options can also be set using environment variables with the following format:
`APP_SECTION__KEY=value`

For example:
- `APP_APPLICATION__PORT=8080`
- `APP_STORAGE__BASE_DIR=/path/to/storage`
- `APP_CACHE__FILESYSTEM__BASE_DIR=/path/to/cache`

Environment variables take precedence over values defined in the configuration files.

### Example: Local Development Configuration

```yaml
application:
  host: 127.0.0.1
  base_url: "http://127.0.0.1"
storage:
  base_dir: /path/to/project
  path_prefix: testdata/
custom_tags:
  env: local
processor:
  max_filter_ops: 256
  concurrency: 1
cache:
  filesystem:
    base_dir: /path/to/project/cache
```

### Example: Production Configuration

```yaml
application:
  host: 0.0.0.0  # Listen on all interfaces
  hmac_secret: ${API_HMAC_SECRET}  # Use environment variable for secret
storage:
  client:
    S3:
      region: ${AWS_REGION}
      bucket: ${S3_BUCKET}
      access_key: ${AWS_ACCESS_KEY_ID}
      secret_key: ${AWS_SECRET_ACCESS_KEY}
processor:
  concurrency: 8  # Use more concurrent threads in production
cache:
  Redis:
    uri: ${REDIS_URL}
custom_tags:
  env: production
```
