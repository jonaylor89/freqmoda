# FreqModa Development Guide

> **This repository is archived.** FreqModa is an experimental AI audio chat demo built for a [blog post](https://jonaylor.com/blog/building-a-simple-ai-daw-part-2-mcp-and-agents/). The core audio processing server lives at [streaming-engine](https://github.com/jonaylor89/streaming-engine).

## Build & Test Commands
- `just` — List available recipes
- `just dev` — Run dev server with auto-reload
- `just build` — Build for production
- `just check` — Type check

## External Dependencies
- **[streaming-engine](https://github.com/jonaylor89/streaming-engine)** — audio processing server (must be running on port 8080)
- **OpenAI API key** — set `OPENAI_API_KEY` in `.env`

## Project Structure
- `src/routes/` — Pages and API endpoints
- `src/lib/` — Shared modules (types, samples, streaming engine client)
- `scripts/` — Development utilities

## Code Style
- **Framework**: SvelteKit 2 with Svelte 5 (runes)
- **Styling**: Tailwind CSS v4
- **TypeScript**: Strict mode
- **State**: Svelte 5 runes (`$state`, `$derived`, `$props`)
