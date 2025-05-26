# Encoded Parameters Feature

This document describes the encoded parameters feature that allows for compact representation of complex audio processing pipelines in URLs.

## Overview

The encoded parameters feature allows you to encode complex parameter sets into a compact base64 string, reducing URL complexity and improving parameter management. Instead of having long URLs with many query parameters, you can encode them into a single `encoded` parameter.

## Usage

### Basic Encoding

```rust
use streaming_engine::streamingpath::params::Params;
use streaming_engine::blob::AudioFormat;

// Create complex parameters
let params = Params {
    key: "track.mp3".to_string(),
    format: Some(AudioFormat::Wav),
    volume: Some(0.8),
    reverse: Some(true),
    lowpass: Some(5000.0),
    ..Default::default()
};

// Encode to compact string
let encoded = params.encode().unwrap();
// Result: "eyJrZXkiOiJ0cmFjay5tcDMiLCJmb3JtYXQiOiJ3YXYi..."
```

### URL Format Examples

#### Traditional Format
```
/track.mp3?format=wav&volume=0.8&reverse=true&lowpass=5000&sample_rate=48000&channels=2
```

#### Encoded Format
```
/track.mp3?encoded=eyJrZXkiOiJ0cmFjay5tcDMiLCJmb3JtYXQiOiJ3YXYiLCJ2b2x1bWUiOjAuOCwicmV2ZXJzZSI6dHJ1ZSwibG93cGFzcyI6NTAwMC4wfQ
```

#### Mixed Format (Explicit Parameters Override Encoded)
```
/track.mp3?encoded=eyJrZXkiOiJ0cmFjay5tcDMi...&format=flac&volume=0.5
```

In the mixed format:
- `format=flac` overrides any format in the encoded parameters
- `volume=0.5` overrides any volume in the encoded parameters
- All other parameters from the encoded string are preserved

## Parameter Precedence

When both encoded and explicit parameters are present:

1. **Path key always wins**: The filename in the URL path is always used as the key
2. **Explicit parameters override encoded**: Any explicit query parameter takes precedence over the same parameter in the encoded set
3. **Encoded parameters fill gaps**: Parameters only present in the encoded set are preserved
4. **Tags are merged**: Tag collections are merged with explicit tags overriding encoded ones

## API Methods

### `Params::encode() -> Result<String>`
Encodes the current parameters to a base64 string.

```rust
let encoded = params.encode()?;
```

### `Params::decode(encoded: &str) -> Result<Params>`
Decodes a base64 string back to parameters.

```rust
let params = Params::decode("eyJrZXkiOiJ0cmFjay5tcDMi...")?;
```

### `Params::merge_with(other: Params) -> Params`
Merges two parameter sets with the `other` taking precedence.

```rust
let merged = base_params.merge_with(explicit_params);
```

## Benefits

### 1. Structured Parameter Management
- Parameters are treated as atomic units
- Easier to generate complex parameter sets programmatically
- Version control friendly (encode entire parameter sets)

### 2. Better Cacheability
- Single parameter key instead of many individual ones
- Cache keys are more consistent and predictable
- Reduced cache fragmentation

### 3. Reduced Parsing Complexity
- Server only needs to handle one encoded parameter plus a few explicit ones
- Less URL parsing overhead
- Fewer edge cases in parameter validation

### 4. Programmatic Generation
- Easy to generate and modify complex parameter sets in code
- Better for API clients and automated systems
- Supports complex nested structures (custom filters, tags)

## Implementation Details

### Encoding Format
- Parameters are serialized to JSON using serde
- JSON is then encoded to URL-safe base64 (no padding)
- Uses the `base64::engine::general_purpose::URL_SAFE_NO_PAD` encoder

### URL Parameter Name
The encoded parameters use the query parameter name `encoded`.

### Error Handling
- Invalid base64 strings return descriptive errors
- Invalid JSON content returns parsing errors
- Malformed parameters are handled gracefully

## Examples

### Simple Audio Processing
```rust
let params = Params {
    key: "song.mp3".to_string(),
    format: Some(AudioFormat::Flac),
    volume: Some(0.9),
    ..Default::default()
};
let encoded = params.encode()?;
// URL: /song.mp3?encoded=eyJrZXkiOiJzb25nLm1wMyI...
```

### Complex Processing Pipeline
```rust
let params = Params {
    key: "track.mp3".to_string(),
    format: Some(AudioFormat::Wav),
    sample_rate: Some(48000),
    channels: Some(2),
    volume: Some(0.85),
    normalize: Some(true),
    lowpass: Some(8000.0),
    highpass: Some(80.0),
    fade_in: Some(1.5),
    fade_out: Some(2.0),
    custom_filters: Some(vec![
        "vibrato=f=5:d=0.5".to_string(),
        "chorus=0.5:0.9:50:0.4:0.25:2".to_string(),
    ]),
    ..Default::default()
};
let encoded = params.encode()?;
// Much more compact than the equivalent traditional URL
```

### Override Specific Parameters
```rust
// Start with encoded complex pipeline
let base_url = "/track.mp3?encoded=eyJmb3JtYXQiOiJ3YXYi...";

// Override specific parameters
let final_url = format!("{}&format=flac&volume=0.7", base_url);
// Result: format=flac and volume=0.7 override encoded values
```

## Testing

The feature includes comprehensive tests covering:
- Basic encoding/decoding round trips
- Parameter merging behavior
- URL parsing with mixed parameters
- Error handling for invalid input
- Integration with existing route handlers

Run tests with:
```bash
cargo test streamingpath::params::tests
```

## Backwards Compatibility

This feature is fully backwards compatible:
- Existing URLs without encoded parameters continue to work
- Traditional query parameters are still supported
- No changes required to existing client code
- Can be adopted incrementally