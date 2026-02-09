# Closed-Loop Audio Iteration

## Overview

FreqModa's core differentiator: an agentic audio system where the AI can **perceive → reason → act → verify → iterate** on audio, rather than blindly applying effects in a single shot.

No existing tool — DAW, AI generator, or audio plugin — does this autonomously. Suno and Udio are slot machines (pull the lever, hope for the best). DAWs require a human in every loop. FreqModa closes the loop by giving the agent ears.

## The Loop

```
┌──────────────────────────────────────────────────┐
│                                                  │
│   ┌──────────┐    ┌──────────┐    ┌──────────┐   │
│   │          │    │          │    │          │   │
│   │ PERCEIVE ├───►│  REASON  ├───►│   ACT    │   │
│   │          │    │          │    │          │   │
│   └──────────┘    └──────────┘    └─────┬────┘   │
│        ▲                                │        │
│        │                                ▼        │
│   ┌────┴─────┐                   ┌──────────┐   │
│   │          │                   │          │   │
│   │ ITERATE  │◄──────────────────┤  VERIFY  │   │
│   │          │                   │          │   │
│   └──────────┘                   └──────────┘   │
│                                                  │
└──────────────────────────────────────────────────┘
```

### 1. PERCEIVE — What am I working with?

The agent analyzes the source audio using two complementary systems:

**Deterministic analysis (Rust-native DSP, streaming engine):**
- EBU R128 loudness (integrated LUFS, true peak, loudness range)
- BPM / tempo via onset autocorrelation
- Musical key via chromagram correlation
- Spectral features (centroid, rolloff, flatness)
- Onset / transient positions
- Silence detection
- RMS, peak, crest factor, dynamic range

**Semantic analysis (ACE-Step DCAE on Modal, optional):**
- DCAE latent embedding (8×16×T tensor capturing musical structure)
- MERT embeddings (768-dim, 75Hz — general musical features)
- mHuBERT embeddings (768-dim, 50Hz — vocal/lyric content)

The deterministic layer tells the agent *what* is in the audio. The semantic layer tells the agent *what it means*.

### 2. REASON — What should I do?

The LLM receives structured analysis JSON and the user's intent. It decides on a plan:

```
User: "Make this sound wider and more atmospheric"

Agent reasoning (internal):
- Analysis shows: mono signal, spectral centroid at 4200Hz (bright),
  no reverb detected, loudness at -14 LUFS
- Plan: add stereo widening via chorus, lower highpass to let in
  more low-mid warmth, add medium echo for space, target -14 LUFS
  after processing to maintain loudness
```

The agent's plan is grounded in measurement, not guessing.

### 3. ACT — Execute the plan

The agent calls the streaming engine's DSP pipeline:

```
POST /api/audio/process
{
  "audio_name": "sample3.mp3",
  "parameters": {
    "chorus": "medium",
    "echo": "medium",
    "highpass": 60.0,
    "bass": 2.0
  }
}
```

Or for generation tasks, calls ACE-Step on Modal:

```
POST /modal/generate
{
  "tags": "atmospheric, wide stereo, ambient pad, Fm, 124bpm",
  "duration": 30.0
}
```

### 4. VERIFY — Did it work?

The agent re-analyzes the output using the same perception pipeline:

```
Agent verification:
- Loudness: -14.2 LUFS → -15.1 LUFS (dropped 0.9 — acceptable)
- Spectral centroid: 4200Hz → 3100Hz (warmer — good)
- Stereo width: mono → stereo (chorus applied — good)
- Echo present: yes (verified)
- BUT: low-end energy increased too much, bass boost at 60Hz
  is causing muddiness (spectral rolloff dropped from 8200 to 4800)
```

### 5. ITERATE — Fix what's wrong

The agent identifies the problem and adjusts:

```
Agent iteration:
- Problem: bass boost too aggressive at 60Hz
- Fix: reduce bass boost, add highpass at 80Hz to clean up sub-bass
- Re-process with adjusted parameters
- Re-verify: spectral rolloff back to 6800 — acceptable
- Loudness: -14.8 LUFS — within 1 LU of target
- Done.
```

The loop continues until the agent's verification passes its quality criteria or a maximum iteration count is reached.

## Architecture

### Two Branches, Composable

The analysis and generation capabilities are developed on two independent branches that compose together:

**`cyanite` branch — Deterministic Audio Analysis**
- Rust-native DSP crate (`crates/audio-analysis/`)
- No GPU, no models, no external dependencies beyond `rustfft`
- Runs in the streaming engine process
- Provides: BPM, key, loudness, spectrum, onsets, dynamics, silence
- New endpoint: `GET /analyze/unsafe/{key}`
- New LLM tool: `analyze_audio`

**`acestep` branch — Neural Audio Intelligence + Generation**
- ACE-Step running on Modal.com (serverless GPU)
- Provides: DCAE latent encoding, MERT/mHuBERT embeddings, music generation, audio2audio style transfer, repainting, variations
- Streaming engine proxies requests to Modal
- New endpoints: `POST /encode/{key}`, `POST /generate`, `POST /transform`
- New LLM tools: `get_audio_embeddings`, `generate_audio`, `transform_audio`, `compare_audio`

When both branches are merged, the agent has full closed-loop capability: deterministic analysis for precise measurement, neural embeddings for semantic understanding, DSP for surgical processing, and generation for creating new material.

### Tool Definitions for the Agent

The LLM's tool set expands to support the full loop:

| Tool | Source | Purpose in Loop |
|------|--------|-----------------|
| `analyze_audio` | cyanite | **PERCEIVE** — deterministic measurement |
| `get_audio_embeddings` | acestep | **PERCEIVE** — semantic understanding |
| `compare_audio` | acestep | **VERIFY** — similarity checking |
| `process_audio` | existing | **ACT** — DSP effects chain |
| `generate_audio` | acestep | **ACT** — create new audio |
| `transform_audio` | acestep | **ACT** — style transfer |
| `list_audio_samples` | existing | Utility |

### Verification Criteria

The agent uses measurable criteria to decide if an iteration succeeded:

- **Loudness**: within ±1 LU of target
- **BPM**: within ±0.5 BPM of source (if tempo should be preserved)
- **Key**: matches source or target key
- **Spectral balance**: centroid/rolloff within expected range for the target style
- **Dynamic range**: crest factor appropriate for genre
- **Embedding similarity**: cosine similarity to reference above threshold (for style-matching tasks)

These are not hardcoded — the agent determines which criteria matter based on the user's request.

### Iteration Limits

To prevent infinite loops:

- **Max iterations**: 5 per user request (configurable)
- **Convergence check**: if the analysis delta between iterations is below a threshold, stop (the changes aren't improving anything)
- **User budget**: each iteration consumes compute; track and limit per-session

## Example Flows

### Flow 1: "Make this brighter and more energetic"

```
Iteration 0: PERCEIVE
  → analyze_audio("sample3.mp3")
  → spectral_centroid: 2100Hz, loudness: -16 LUFS, bpm: 98

Iteration 1: ACT + VERIFY
  → process_audio(treble: +3dB, speed: 1.05, volume: 1.1)
  → analyze_audio(result)
  → spectral_centroid: 3400Hz ✓, loudness: -13.8 LUFS (too hot), bpm: 102.9

Iteration 2: ACT + VERIFY
  → process_audio(result, volume: 0.9, normalize: true, normalize_level: -14)
  → analyze_audio(result)
  → spectral_centroid: 3400Hz ✓, loudness: -14.1 LUFS ✓, bpm: 102.9 ✓
  → DONE
```

### Flow 2: "Generate something that matches this vibe"

```
Iteration 0: PERCEIVE
  → analyze_audio("reference.mp3")
  → bpm: 124, key: Fm, centroid: 2800 (dark), loudness: -12 LUFS
  → get_audio_embeddings("reference.mp3")
  → reference_embedding: [...]

Iteration 1: ACT + VERIFY
  → generate_audio(tags: "dark electronic, Fm, 124bpm, moody synths")
  → analyze_audio(generated)
  → bpm: 123.5 ✓, key: Fm ✓, centroid: 3900 (too bright)
  → compare_audio(reference, generated)
  → similarity: 0.71 (below 0.80 threshold)

Iteration 2: ACT + VERIFY
  → generate_audio(tags: "dark electronic, Fm, 124bpm, low-fi, warm, muted highs")
  → analyze_audio(generated)
  → bpm: 124.1 ✓, key: Fm ✓, centroid: 2650 ✓
  → compare_audio(reference, generated)
  → similarity: 0.84 ✓
  → DONE
```

### Flow 3: "Fix the muddiness in the low end"

```
Iteration 0: PERCEIVE
  → analyze_audio("mix.wav")
  → bass_energy: excessive below 200Hz, spectral_rolloff: 3200Hz (very low)
  → dynamic_range: 6dB (over-compressed)

Iteration 1: ACT + VERIFY
  → process_audio(highpass: 40, lowpass: null, bass: -3, compressor: "2:1:10:50:200")
  → analyze_audio(result)
  → bass_energy: reduced ✓, spectral_rolloff: 5800Hz ✓
  → dynamic_range: 9dB (better but could improve)

Iteration 2: ACT + VERIFY
  → process_audio(result, normalize: true, normalize_level: -14)
  → analyze_audio(result)
  → loudness: -14.0 LUFS ✓, dynamic_range: 9dB ✓
  → DONE
```

## What This Enables

- **Sound design at scale**: generate 200 footstep variations, verify each meets spectral/loudness criteria
- **Adaptive music systems**: generate stems, verify harmonic compatibility via key detection + embedding similarity
- **Reference matching**: encode reference → generate → compare → iterate until vibe matches
- **Automated mastering feedback**: analyze a mix, identify problems (too bright, too compressed, phase issues), suggest or apply fixes, verify improvement
- **Quality assurance pipelines**: batch-process a catalog, flag tracks that don't meet loudness/spectral standards

## Non-Goals

- Replacing a human mix engineer's subjective taste
- Real-time processing (the loop is deliberate, not instantaneous)
- Pixel-perfect reproduction of a reference (similarity threshold, not identity)
