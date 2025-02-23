'use client';

import { useParams, useRouter } from 'next/navigation';
import { useState, useEffect } from 'react';
import tracksData from "@/data/tracks.json";
import WaveformVisualizer from '@/components/WaveformVisualizer';
import VolumeControl from '@/components/VolumeControl';

export default function EditTrack() {
  const params = useParams();
  const router = useRouter();
  const [duration, setDuration] = useState("0:00");
  const [volume, setVolume] = useState(1);

  useEffect(() => {
    return () => {
      setIsPlaying(false);
    };
  }, []);

  const track = tracksData.tracks.find(t => t.id === Number(params.id));
  const [isPlaying, setIsPlaying] = useState(false);
  const [isPlaybackReady, setIsPlaybackReady] = useState(false);

  const formatTime = (seconds: number) => {
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = Math.floor(seconds % 60);
    return `${minutes}:${remainingSeconds.toString().padStart(2, '0')}`;
  };

  if (!track) return null;

  const togglePlayback = () => {
    setIsPlaying(!isPlaying);
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-[#13111C] to-[#191627] text-white p-8">
      <div className="max-w-4xl mx-auto">
        <div className="flex items-center gap-8 mb-1 py-4">
          <button 
            onClick={() => router.back()}
            className="text-[#B4A5FF] hover:text-white transition-colors mb-12 flex items-center gap-2"
          >
            <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
              <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"/>
            </svg>
            Back
          </button>
          <h1 className="text-5xl font-bold bg-gradient-to-r from-[#B4A5FF] to-[#7C66FF] bg-clip-text text-transparent leading-relaxed">
            {track.name}
          </h1>
        </div>

        <div className="bg-[#1C1A27]/70 backdrop-blur-xl rounded-3xl p-8 shadow-2xl border border-white/5">
          <div className="mb-8">
            <WaveformVisualizer 
              trackId={track.id} 
              audioUrl={track.audioUrl}
              onPlaybackReady={(audioDuration) => {
                setIsPlaybackReady(true);
                setDuration(formatTime(audioDuration));
              }}
              isPlaying={isPlaying}
              onPlayPause={togglePlayback}
              volume={volume}
              onVolumeChange={setVolume}
            />
          </div>

          <div className="flex items-center justify-between mb-8">
            <div className="flex items-center gap-4">
              <button 
                onClick={togglePlayback}
                disabled={!isPlaybackReady}
                className="p-3 rounded-full bg-[#B4A5FF] text-white hover:bg-[#9785FF] transition-colors disabled:opacity-50"
              >
                {isPlaying ? <PauseIcon /> : <PlayIcon />}
              </button>
              <span className="font-mono">{duration}</span>
            </div>
            <div className="flex items-center gap-2">
              <VolumeControl volume={volume} onVolumeChange={setVolume} />
              <button className="p-2 rounded-lg bg-[#B4A5FF]/10 text-[#B4A5FF]">
                <FullscreenIcon />
              </button>
            </div>
          </div>

          <div className="space-y-4">
            <h2 className="text-xl font-semibold mb-4">Quick Effects</h2>
            <div className="flex flex-wrap gap-3">
              {[
                "slow and reverb the song",
                "reverse the first 5 seconds",
                "speed up the chorus by 1.5x",
                "add echo effect to the vocals"
              ].map((effect, i) => (
                <button
                  key={i}
                  className="px-4 py-2 rounded-full bg-[#B4A5FF]/10 text-[#B4A5FF] hover:bg-[#B4A5FF]/20 transition-colors"
                >
                  {effect}
                </button>
              ))}
            </div>
          </div>

          <div className="mt-8">
            <div className="relative">
              <input
                type="text"
                placeholder="Type a command..."
                className="w-full bg-[#13111C] border border-white/10 rounded-xl px-4 py-3 pr-24 focus:outline-none focus:border-[#B4A5FF]/50"
              />
              <div className="absolute right-3 top-1/2 -translate-y-1/2 flex items-center gap-2">
                <span className="text-sm text-gray-400">0s - 30s</span>
                <button className="p-2 rounded-lg bg-[#B4A5FF] text-white">
                  <SendIcon />
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

const PlayIcon = () => (
  <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
    <path d="M8 5v14l11-7z" />
  </svg>
);

const PauseIcon = () => (
  <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
    <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z" />
  </svg>
);

const FullscreenIcon = () => (
  <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
    <path d="M7 14H5v5h5v-2H7v-3zm-2-4h2V7h3V5H5v5zm12 7h-3v2h5v-5h-2v3zM14 5v2h3v3h2V5h-5z" />
  </svg>
);

const SendIcon = () => (
  <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
    <path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z" />
  </svg>
); 