
# FreqModa (Archived)

> ⚠️ **This repository is archived.** The streaming engine — the core audio processing server — has moved to [**streaming-engine**](https://github.com/jonaylor89/streaming-engine).

FreqModa is an experimental AI-powered audio chat application built as a companion to [this blog post](https://jonaylor.com/blog/building-a-simple-ai-daw-part-2-mcp-and-agents/). It uses LLMs to orchestrate audio processing via natural language.

👉 **[github.com/jonaylor89/streaming-engine](https://github.com/jonaylor89/streaming-engine)** — On-the-fly audio processing server (like Thumbor, but for audio)

[![Create a Simple Ai DAW, Part 2: MCP and Agents](https://img.youtube.com/vi/BGSBQDlbwb8/hqdefault.jpg)](https://www.youtube.com/watch?v=BGSBQDlbwb8)   

## Prerequisites

- Node.js 20+
- A running [streaming-engine](https://github.com/jonaylor89/streaming-engine) instance on port 8080
- An OpenAI API key

## Quick Start

```sh
cp .env.example .env   # Add your OPENAI_API_KEY
npm install
npm run dev
```
