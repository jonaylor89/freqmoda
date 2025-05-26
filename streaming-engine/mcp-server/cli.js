#!/usr/bin/env node

import { spawn } from 'child_process';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const serverScript = join(__dirname, 'index.js');

// Parse command line arguments
const args = process.argv.slice(2);
const serverUrl = args.find(arg => arg.startsWith('--server='))?.split('=')[1] || 'http://localhost:8080';

// Set environment variable for server URL
process.env.STREAMING_ENGINE_SERVER_URL = serverUrl;

console.error(`🎵 Starting Streaming Engine MCP Server`);
console.error(`🔗 Connecting to streaming engine at: ${serverUrl}`);
console.error(`📡 MCP server ready for LLM connections...`);

// Start the MCP server
const child = spawn('node', [serverScript], {
  stdio: ['inherit', 'inherit', 'inherit'],
  env: process.env
});

child.on('exit', (code) => {
  process.exit(code);
});

// Handle signals
process.on('SIGINT', () => {
  child.kill('SIGINT');
});

process.on('SIGTERM', () => {
  child.kill('SIGTERM');
});