#!/usr/bin/env node

import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
} from "@modelcontextprotocol/sdk/types.js";
import axios from "axios";

// Default server configuration
const DEFAULT_SERVER_URL = process.env.STREAMING_ENGINE_SERVER_URL || "http://localhost:8080";

class StreamingEngineMCPServer {
  constructor() {
    this.server = new Server(
      {
        name: "streaming-engine-audio-processor",
        version: "1.0.0",
      },
      {
        capabilities: {
          tools: {},
        },
      }
    );

    this.setupToolHandlers();
  }

  setupToolHandlers() {
    this.server.setRequestHandler(ListToolsRequestSchema, async () => {
      return {
        tools: [
          {
            name: "process_audio",
            description: "Process audio with various effects and transformations",
            inputSchema: {
              type: "object",
              properties: {
                audio_name: {
                  type: "string",
                  description: "URL/URI/filename to the audio file to process",
                },
                format: {
                  type: "string",
                  description: "Output format (mp3, wav, etc.)",
                  enum: ["mp3", "wav", "flac", "ogg", "m4a"],
                },
                // Time operations
                start_time: {
                  type: "number",
                  description: "Start time in seconds",
                },
                duration: {
                  type: "number",
                  description: "Duration in seconds",
                },
                speed: {
                  type: "number",
                  description: "Playback speed multiplier (e.g., 0.5 = half speed, 2.0 = double speed)",
                },
                reverse: {
                  type: "boolean",
                  description: "Reverse the audio",
                },
                // Volume operations
                volume: {
                  type: "number",
                  description: "Volume adjustment multiplier (1.0 = original, 0.5 = half volume, 2.0 = double volume)",
                },
                normalize: {
                  type: "boolean",
                  description: "Normalize audio levels",
                },
                normalize_level: {
                  type: "number",
                  description: "Target normalization level in dB",
                },
                // Filters
                lowpass: {
                  type: "number",
                  description: "Lowpass filter cutoff frequency in Hz",
                },
                highpass: {
                  type: "number",
                  description: "Highpass filter cutoff frequency in Hz",
                },
                bass: {
                  type: "number",
                  description: "Bass boost/cut level in dB",
                },
                treble: {
                  type: "number",
                  description: "Treble boost/cut level in dB",
                },
                // Fades
                fade_in: {
                  type: "number",
                  description: "Fade in duration in seconds",
                },
                fade_out: {
                  type: "number",
                  description: "Fade out duration in seconds",
                },
                // Advanced effects (use built-in presets only)
                echo: {
                  type: "string",
                  description: "Echo effect - use simple values like 'light', 'medium', or 'heavy'. Avoid complex FFmpeg syntax.",
                },
                chorus: {
                  type: "string", 
                  description: "Chorus effect - use simple values like 'light', 'medium', or 'heavy'. Avoid complex FFmpeg syntax.",
                },
                flanger: {
                  type: "string",
                  description: "Flanger effect - use simple values like 'light', 'medium', or 'heavy'. Avoid complex FFmpeg syntax.",
                },
              },
              required: ["audio_name"],
            },
          },
          {
            name: "preview_audio_params",
            description: "Preview the parameters that would be used for audio processing without actually processing",
            inputSchema: {
              type: "object",
              properties: {
                audio_name: {
                  type: "string",
                  description: "URI/URL/filename to the audio file",
                },
                format: { type: "string" },
                start_time: { type: "number" },
                duration: { type: "number" },
                speed: { type: "number" },
                reverse: { type: "boolean" },
                volume: { type: "number" },
                normalize: { type: "boolean" },
                normalize_level: { type: "number" },
                lowpass: { type: "number" },
                highpass: { type: "number" },
                bass: { type: "number" },
                treble: { type: "number" },
                fade_in: { type: "number" },
                fade_out: { type: "number" },
                echo: { type: "string" },
                chorus: { type: "string" },
                flanger: { type: "string" },
              },
              required: ["audio_name"],
            },
          },
          {
            name: "get_audio_metadata",
            description: "Extract metadata from an audio file including format, duration, bitrate, sample rate, channels, codec, and ID3 tags",
            inputSchema: {
              type: "object",
              properties: {
                audio_name: {
                  type: "string",
                  description: "URI/URL/filename to the audio file to analyze",
                },
              },
              required: ["audio_name"],
            },
          },
          {
            name: "get_server_health",
            description: "Check if the streaming engine audio server is running and healthy",
            inputSchema: {
              type: "object",
              properties: {},
            },
          },
        ],
      };
    });

    this.server.setRequestHandler(CallToolRequestSchema, async (request) => {
      const { name, arguments: args } = request.params;

      try {
        switch (name) {
          case "process_audio":
            return await this.processAudio(args);
          case "preview_audio_params":
            return await this.previewParams(args);
          case "get_audio_metadata":
            return await this.getAudioMetadata(args);
          case "get_server_health":
            return await this.getServerHealth();
          default:
            throw new Error(`Unknown tool: ${name}`);
        }
      } catch (error) {
        return {
          content: [
            {
              type: "text",
              text: `Error: ${error.message}`,
            },
          ],
          isError: true,
        };
      }
    });
  }

  buildQueryParams(args) {
    const params = new URLSearchParams();
    
    // Preset mappings for common effect levels
    const effectPresets = {
      echo: {
        light: "0.6:0.3:1000:0.3",
        medium: "0.8:0.88:60:0.4", 
        heavy: "0.8:0.9:1000:0.5"
      },
      chorus: {
        light: "0.5:0.9:50:0.4:0.25:2",
        medium: "0.7:0.9:50:0.4:0.25:2",
        heavy: "0.9:0.9:50:0.4:0.25:2"
      },
      flanger: {
        light: "0.5:0.75:2:0.25:2",
        medium: "0.7:0.75:3:0.25:2", 
        heavy: "0.9:0.75:4:0.25:2"
      }
    };
    
    // Filter out audio_name, custom_filters, custom_options and undefined values
    Object.entries(args).forEach(([key, value]) => {
      if (key !== "audio_name" && 
          key !== "custom_filters" && 
          key !== "custom_options" && 
          value !== undefined && 
          value !== null) {
        
        // Map preset values for effects
        if (effectPresets[key] && effectPresets[key][value.toLowerCase()]) {
          params.append(key, effectPresets[key][value.toLowerCase()]);
        } else {
          params.append(key, value.toString());
        }
      }
    });
    
    return params.toString();
  }

  async processAudio(args) {
    const { audio_name } = args;
    const queryParams = this.buildQueryParams(args);
    const url = `${DEFAULT_SERVER_URL}/unsafe/${encodeURIComponent(audio_name)}${queryParams ? `?${queryParams}` : ""}`;

    try {
      const response = await axios.get(url, {
        responseType: 'arraybuffer',
        timeout: 30000,
      });

      const contentType = response.headers['content-type'] || 'audio/mpeg';
      const audioData = Buffer.from(response.data);
      
      return {
        content: [
          {
            type: "text",
            text: `Successfully processed audio from ${audio_name}. Output size: ${audioData.length} bytes, Content-Type: ${contentType}`,
          },
          {
            type: "text",
            text: `Processed audio URL: ${url}`,
          },
        ],
      };
    } catch (error) {
      throw new Error(`Failed to process audio: ${error.message}`);
    }
  }

  async previewParams(args) {
    const { audio_name } = args;
    const queryParams = this.buildQueryParams(args);
    const url = `${DEFAULT_SERVER_URL}/params/unsafe/${encodeURIComponent(audio_name)}${queryParams ? `?${queryParams}` : ""}`;

    try {
      const response = await axios.get(url);
      
      return {
        content: [
          {
            type: "text",
            text: `Parameter preview for ${audio_name}:\n\`\`\`json\n${JSON.stringify(response.data, null, 2)}\n\`\`\``,
          },
        ],
      };
    } catch (error) {
      throw new Error(`Failed to preview parameters: ${error.message}`);
    }
  }

  async getAudioMetadata(args) {
    const { audio_name } = args;
    const url = `${DEFAULT_SERVER_URL}/meta/unsafe/${encodeURIComponent(audio_name)}`;

    try {
      const response = await axios.get(url, {
        timeout: 15000,
      });
      
      const metadata = response.data;
      
      // Format the metadata nicely for display
      let metadataText = `Audio metadata for ${audio_name}:\\n\\n`;
      metadataText += `📄 **Format**: ${metadata.format}\\n`;
      metadataText += `⏱️ **Duration**: ${metadata.duration ? metadata.duration.toFixed(2) + ' seconds' : 'Unknown'}\\n`;
      metadataText += `📊 **Sample Rate**: ${metadata.sample_rate ? metadata.sample_rate + ' Hz' : 'Unknown'}\\n`;
      metadataText += `🔊 **Channels**: ${metadata.channels || 'Unknown'}\\n`;
      metadataText += `🎵 **Codec**: ${metadata.codec || 'Unknown'}\\n`;
      metadataText += `💾 **Bit Rate**: ${metadata.bit_rate ? metadata.bit_rate + ' bps' : 'Unknown'}\\n`;
      metadataText += `📦 **File Size**: ${metadata.size ? (metadata.size / 1024 / 1024).toFixed(2) + ' MB' : 'Unknown'}\\n`;
      
      if (metadata.tags && Object.keys(metadata.tags).length > 0) {
        metadataText += `\\n🏷️ **Tags**:\\n`;
        for (const [key, value] of Object.entries(metadata.tags)) {
          metadataText += `  • **${key}**: ${value}\\n`;
        }
      }
      
      return {
        content: [
          {
            type: "text",
            text: metadataText,
          },
          {
            type: "text",
            text: `\\n**Raw metadata JSON**:\\n\`\`\`json\\n${JSON.stringify(metadata, null, 2)}\\n\`\`\``,
          },
        ],
      };
    } catch (error) {
      throw new Error(`Failed to get audio metadata: ${error.message}`);
    }
  }

  async getServerHealth() {
    try {
      const response = await axios.get(`${DEFAULT_SERVER_URL}/health`, {
        timeout: 5000,
      });
      
      return {
        content: [
          {
            type: "text",
            text: `Server is healthy. Status: ${response.status}`,
          },
        ],
      };
    } catch (error) {
      throw new Error(`Server health check failed: ${error.message}`);
    }
  }

  async run() {
    const transport = new StdioServerTransport();
    await this.server.connect(transport);
    console.error("Streaming Engine MCP server running on stdio");
  }
}

// Only start the server if this file is run directly (not imported/checked)
if (import.meta.url === `file://${process.argv[1]}`) {
  const server = new StreamingEngineMCPServer();
  server.run().catch(console.error);
}