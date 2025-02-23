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
  const [currentTime, setCurrentTime] = useState(0);
  const [duration, setDuration] = useState(0);

  useEffect(() => {
    if (!containerRef.current) return;

    const wavesurfer = WaveSurfer.create({
      container: containerRef.current,
      waveColor: '#B4A5FF',
      progressColor: '#7C66FF',
      height: 160,
      normalize: true,
      backend: 'WebAudio',
      cursorWidth: 0,
      barWidth: 3,
      barGap: 2,
      minPxPerSec: 30,
      maxCanvasWidth: 4000,
      interact: false,
      dragToSeek: false,
      scrollParent: false, // Disable scrolling
    });

    // Add time update handler
    wavesurfer.on('timeupdate', (currentTime) => {
      setCurrentTime(currentTime);
    });

    wavesurfer.on('ready', () => {
      setLoading(false);
      setError(null);
      setDuration(wavesurfer.getDuration());
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

  const handleTimelineClick = (e: React.MouseEvent<HTMLDivElement>) => {
    const wavesurfer = wavesurferRef.current;
    if (!wavesurfer || loading) return;

    const timeline = e.currentTarget;
    const rect = timeline.getBoundingClientRect();
    const relativeX = e.clientX - rect.left;
    const percentage = relativeX / rect.width;
    const seekTime = duration * percentage;
    
    wavesurfer.seekTo(percentage);
    setCurrentTime(seekTime);
  };

  const formatTime = (time: number) => {
    const minutes = Math.floor(time / 60);
    const seconds = Math.floor(time % 60);
    return `${minutes}:${seconds.toString().padStart(2, '0')}`;
  };

  return (
    <div className="w-full rounded-xl bg-[#13111C]/50 relative space-y-2 overflow-hidden">
      <div 
        ref={containerRef} 
        className="h-40" 
        style={{ 
          overflow: 'hidden',
          touchAction: 'none', // Disable touch scrolling
          pointerEvents: isPlaying ? 'none' : 'auto' // Disable interaction while playing
        }} 
      />
      
      {/* Enhanced Timeline with Mini Waveform */}
      <div className="px-4 pb-4">
        <div 
          className="h-12 bg-[#13111C] rounded-xl cursor-pointer relative group overflow-hidden"
          onClick={handleTimelineClick}
          onMouseMove={(e) => {
            const rect = e.currentTarget.getBoundingClientRect();
            const percentage = ((e.clientX - rect.left) / rect.width) * 100;
            e.currentTarget.style.setProperty('--hover-position', `${percentage}%`);
          }}
        >
          {/* Mini waveform background */}
          <div 
            className="absolute inset-0 opacity-30"
            style={{
              backgroundImage: 'repeating-linear-gradient(90deg, #B4A5FF 0px, #B4A5FF 2px, transparent 2px, transparent 4px)',
              backgroundSize: '100% 100%',
              maskImage: 'linear-gradient(90deg, transparent, #000 10%, #000 90%, transparent)',
              WebkitMaskImage: 'linear-gradient(90deg, transparent, #000 10%, #000 90%, transparent)',
            }}
          />

          {/* Progress gradient overlay */}
          <div 
            className="absolute inset-y-0 left-0 bg-gradient-to-r from-[#7C66FF] to-[#B4A5FF]"
            style={{ 
              width: `${(currentTime / duration) * 100}%`,
              opacity: 0.3,
            }}
          />

          {/* Active progress bar */}
          <div 
            className="absolute bottom-0 left-0 h-1.5 bg-gradient-to-r from-[#7C66FF] to-[#B4A5FF]"
            style={{ 
              width: `${(currentTime / duration) * 100}%`,
              boxShadow: '0 0 20px rgba(124, 102, 255, 0.5)',
            }}
          />

          {/* Hover indicator */}
          <div 
            className="absolute inset-y-0 w-0.5 bg-white opacity-0 group-hover:opacity-50 transition-opacity"
            style={{ 
              left: 'var(--hover-position, 0%)',
              boxShadow: '0 0 10px rgba(255, 255, 255, 0.5)',
            }}
          />

          {/* Time marker */}
          <div 
            className="absolute bottom-0 h-12 w-1 bg-white"
            style={{ 
              left: `${(currentTime / duration) * 100}%`,
              transform: 'translateX(-50%)',
              opacity: 0.8,
              boxShadow: '0 0 15px rgba(255, 255, 255, 0.3)',
            }}
          >
            {/* Marker glow */}
            <div 
              className="absolute bottom-0 w-3 h-3 -translate-x-1/2 rounded-full"
              style={{
                background: 'radial-gradient(circle, rgba(255,255,255,1) 0%, rgba(255,255,255,0) 70%)',
                filter: 'blur(2px)',
              }}
            />
          </div>
        </div>

        {/* Time display */}
        <div className="flex justify-between mt-3">
          <div className="text-sm font-mono bg-[#13111C] px-3 py-1.5 rounded-lg text-[#B4A5FF] shadow-lg">
            {formatTime(currentTime)}
          </div>
          <div className="text-sm font-mono bg-[#13111C] px-3 py-1.5 rounded-lg text-[#B4A5FF] shadow-lg">
            {formatTime(duration)}
          </div>
        </div>
      </div>

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