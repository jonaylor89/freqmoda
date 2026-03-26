import { env } from '$env/dynamic/private';

function getBaseUrl(): string {
  return env.STREAMING_ENGINE_URL || 'http://localhost:8080';
}

export async function processAudio(audioName: string, params: Record<string, unknown>): Promise<string> {
  const queryParts: string[] = [];
  for (const [key, value] of Object.entries(params)) {
    if (key === 'audio_name') continue;
    if (value !== undefined && value !== null && value !== '') {
      queryParts.push(`${encodeURIComponent(key)}=${encodeURIComponent(String(value))}`);
    }
  }
  const query = queryParts.length > 0 ? `?${queryParts.join('&')}` : '';
  const baseUrl = getBaseUrl();
  const url = `${baseUrl}/unsafe/${encodeURIComponent(audioName)}${query}`;

  // Validate URL by making a HEAD request
  const response = await fetch(url, { method: 'HEAD' });
  if (!response.ok) {
    throw new Error(`Audio processing failed: ${response.status}`);
  }

  return url;
}

export async function getAudioMetadata(audioName: string): Promise<Record<string, unknown>> {
  const baseUrl = getBaseUrl();
  const url = `${baseUrl}/meta/unsafe/${encodeURIComponent(audioName)}`;
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Failed to get metadata: ${response.status}`);
  }
  return response.json();
}

export async function healthCheck(): Promise<boolean> {
  try {
    const baseUrl = getBaseUrl();
    const response = await fetch(`${baseUrl}/health`);
    return response.ok;
  } catch {
    return false;
  }
}

export function getStreamingEngineUrl(): string {
  return getBaseUrl();
}
