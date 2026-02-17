
# TODO

## 🟢 Easy – Drop-in libraries / minimal glue

### Better Time Stretching
* https://github.com/bungee-audio-stretch/bungee

### Basic Audio Analysis with Essentia
* https://essentia.upf.edu/
* Use the "Music Extractor" pipeline to get duration, BPM, key, loudness, spectral features, tonal features, danceability out of the box
* Use Essentia's built-in mood models (`mood_happy`, `mood_sad`, `mood_aggressive`, `mood_relaxed`) to approximate Energetic / Dark / Happy / Sad sliders
* Use `voice_instrumental` model for basic vocal presence detection

### BPM & Beat Tracking with Madmom
* Very accurate BPM, onset, and downbeat tracking
* Useful for finding bar boundaries and timing-based segmentation

### Basic Feature Extraction with librosa
* Tempo/BPM estimation, key estimation (chroma + template matching), onset detection
* MFCCs, spectral centroid, and other features useful as classifier inputs

## 🟡 Medium – Pretrained model integration / some wiring

### Genre & Mood Tagging with MusiCNN / PANNs
* Pretrained neural networks for multi-label music tagging (genres, moods, instruments)
* Outputs tag probabilities → map to scores like "Energetic 0.63, Dark 0.55"
* Essentia already integrates some MusiCNN models as standard algorithms, simplifying deployment
* PANNs (Pretrained Audio Neural Networks, trained on AudioSet) for instrument type and sound event detection

### Emotional Profile / Energy Level
* Use Essentia's valence and arousal models to derive Negative/Positive and Low/High Energy categories
* Aggregate mood probabilities over the full track and threshold into categories
* Alternative: train a small model on public datasets (DEAM, Emomusic) using librosa/Essentia features

### Instrument & Voice Detection via Audio Tagging
* Use PANNs or MusiCNN for frame-level instrument predictions (drums, bass, synth, vocal, guitar, etc.)
* Aggregate across time → "Throughout" (>80% of frames), "Frequently" (40–80%), "Partially" (<40%)

### Structural Segmentation & "Most Representative Segment"
* Use madmom or Essentia for beat tracking, bar segmentation, and structural change detection (timbre/harmony/energy)
* Pick the segment where predicted tags are most stable/confident, or energy is closest to track mean

## 🔴 Hard – Custom models / significant engineering

### Source Separation for Refined Instrument Detection
* Use **Demucs** or **Open-Unmix** to separate stems (vocals, drums, bass, other)
* Measure energy per stem over time for more accurate instrument presence labeling

### Augmented Keyword Generation
* Collect a vocabulary of mood/scene descriptors ("ominous, dark, scary, danger, powerful, menacing…")
* Option A: Train a multilabel classifier on top of MusiCNN/librosa features
* Option B: Use embedding-based similarity with **CLAP** (audio–text model) to match tracks to text descriptors

### Full MIR Pipeline Integration
* Combine all of the above into a unified analysis pipeline:
  1. Essentia + madmom → BPM, key, beats/bars, segments, mood/valence/arousal, danceability
  2. MusiCNN / PANNs → genre, mood, instrument tag probabilities
  3. Demucs/Open-Unmix → refined instrument presence via stem energies
  4. Aggregation layer → genre scores, mood scores (0–1), emotional profile categories, instrument summaries, augmented keywords
* Decide: offline batch processing vs. real-time web API

## 💡 Ideas / Explore

### MusicGen MCP Tool
* Probably using a Udio key

### Cyanite API
* Commercial alternative for audio analysis if self-hosted MIR is too heavy
