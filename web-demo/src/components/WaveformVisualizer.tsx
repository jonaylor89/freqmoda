'use client';

import { useEffect, useRef, useState } from 'react';
import WaveSurfer from 'wavesurfer.js';

interface WaveformVisualizerProps {
  trackId: number;
  audioUrl: string;
  onPlaybackReady?: (duration: number) => void;
  isPlaying?: boolean;
  onPlayPause?: () => void;
}

// Add a LoadingSpinner component
const LoadingSpinner = () => (
  <div className="animate-spin rounded-full h-8 w-8 border-2 border-[#B4A5FF] border-t-transparent" />
);

export default function WaveformVisualizer({ 
  trackId, 
  audioUrl, 
  onPlaybackReady,
  isPlaying,
  onPlayPause 
}: WaveformVisualizerProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const wavesurferRef = useRef<WaveSurfer | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!containerRef.current) return;

    setLoading(true);  // Reset loading state on each load
    setError(null);    // Clear any previous errors

    const wavesurfer = WaveSurfer.create({
      container: containerRef.current,
      waveColor: '#B4A5FF',
      progressColor: '#7C66FF',
      height: 160,
      normalize: true,
      backend: 'WebAudio', // Changed back to WebAudio
      cursorWidth: 2,
      barWidth: 3,
      barGap: 2,
      minPxPerSec: 30,
      maxCanvasWidth: 4000,
      interact: true,
      dragToSeek: true,
    });

    console.log('Loading audio from:', audioUrl);

    wavesurfer.on('ready', () => {
      console.log('WaveSurfer ready');
      setLoading(false);
      setError(null);  // Clear any errors on successful load
      if (onPlaybackReady) {
        onPlaybackReady(wavesurfer.getDuration());
      }
    });

    wavesurfer.on('error', err => {
      console.error('WaveSurfer error:', err);
      setError(`Failed to load audio: ${err.message || 'Unknown error'}`);
      setLoading(false);  // Stop loading on error
    });

    wavesurfer.on('loading', progress => {
      console.log(`Loading: ${progress}%`);
      if (progress < 100) {
        setLoading(true);
      }
    });

    try {
      wavesurfer.load(audioUrl);
      wavesurferRef.current = wavesurfer;

      // Add click handler for play/pause
      containerRef.current.addEventListener('click', () => {
        if (onPlayPause) onPlayPause();
      });
    } catch (err) {
      console.error('Error loading audio:', err);
      setError('Failed to load audio');
      setLoading(false);  // Stop loading on error
    }

    return () => {
      wavesurfer.destroy();
    };
  }, [audioUrl]);

  // Handle play/pause
  useEffect(() => {
    const wavesurfer = wavesurferRef.current;
    if (!wavesurfer) return;
    
    if (isPlaying) {
      wavesurfer.play();
    } else {
      wavesurfer.pause();
    }
  }, [isPlaying]);

  return (
    <div className="w-full rounded-xl bg-[#13111C]/50 relative">
      <div ref={containerRef} className="h-40" />
      {(loading || error) && (
        <div className="absolute inset-0 flex items-center justify-center bg-black/20 backdrop-blur-sm">
          {loading ? (
            <LoadingSpinner />
          ) : error ? (
            <div className="text-red-400">{error}</div>
          ) : null}
        </div>
      )}
    </div>
  );
} 