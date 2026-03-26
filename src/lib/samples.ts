import type { AudioSample } from './types';

export const samples: AudioSample[] = [
  { streamingKey: 'sample1.mp3', title: 'Sample 1', duration: 32.9, fileType: 'audio/mpeg' },
  { streamingKey: 'sample2.mp3', title: 'Sample 2', duration: 32.9, fileType: 'audio/mpeg' },
  { streamingKey: 'sample3.mp3', title: 'Sample 3', duration: 32.9, fileType: 'audio/mpeg' },
  { streamingKey: 'sample4.mp3', title: 'Sample 4', duration: 32.9, fileType: 'audio/mpeg' },
  { streamingKey: 'sample5.mp3', title: 'Sample 5', duration: 130.6, fileType: 'audio/mpeg' },
  { streamingKey: 'sample6.mp3', title: 'Sample 6', duration: 32.9, fileType: 'audio/mpeg' },
  { streamingKey: 'sample7.mp3', title: 'Sample 7', duration: 32.9, fileType: 'audio/mpeg' },
  { streamingKey: 'sample8.mp3', title: 'Sample 8', duration: 32.9, fileType: 'audio/mpeg' },
];

export function findSample(nameOrKey: string): AudioSample | undefined {
  // Exact key match
  const byKey = samples.find(s => s.streamingKey === nameOrKey);
  if (byKey) return byKey;

  // Title match (case insensitive)
  const byTitle = samples.find(s => s.title.toLowerCase() === nameOrKey.toLowerCase());
  if (byTitle) return byTitle;

  // "Sample 1", "sample1" etc.
  const normalized = nameOrKey.toLowerCase();
  if (normalized.startsWith('sample')) {
    const num = normalized.replace(/[^0-9]/g, '');
    if (num) {
      return samples.find(s => s.streamingKey === `sample${num}.mp3`);
    }
  }

  return undefined;
}
