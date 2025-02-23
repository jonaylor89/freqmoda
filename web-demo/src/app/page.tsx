'use client';

import tracksData from "@/data/tracks.json";
import { useRouter } from 'next/navigation';

export default function Home() {
  const router = useRouter();

  return (
    <div className="min-h-screen bg-gradient-to-br from-[#13111C] to-[#191627] text-white p-8">
      <div className="max-w-4xl mx-auto">
        <div className="flex justify-between items-center mb-12">
          <h1 className="text-5xl font-bold animate-gradient-shine from-[#B4A5FF] to-[#7C66FF]">
            Your Tracks
          </h1>
          <button className="text-[#B4A5FF] flex items-center gap-2 px-6 py-3 rounded-xl border border-[#B4A5FF]/20 hover:border-[#B4A5FF]/40 transition-all hover:scale-105 bg-[#B4A5FF]/5 backdrop-blur-sm">
            <svg
              width="24"
              height="24"
              viewBox="0 0 24 24"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
              className="opacity-70"
            >
              <path
                d="M20 4H4C2.9 4 2 4.9 2 6V18C2 19.1 2.9 20 4 20H20C21.1 20 22 19.1 22 18V6C22 4.9 21.1 4 20 4ZM20 18H4V8L12 13L20 8V18ZM12 11L4 6H20L12 11Z"
                fill="currentColor"
              />
            </svg>
            Join the waiting list
          </button>
        </div>

        <div className="bg-[#1C1A27]/70 backdrop-blur-xl rounded-3xl p-8 shadow-2xl border border-white/5">
          <div className="grid grid-cols-[1fr_auto_auto] gap-4 mb-6 px-4">
            <div className="text-gray-400 font-medium uppercase text-sm tracking-wider">Name</div>
            <div className="text-gray-400 font-medium uppercase text-sm tracking-wider">Duration</div>
            <div className="text-gray-400 font-medium uppercase text-sm tracking-wider">Actions</div>
          </div>

          {tracksData.tracks.map((track) => (
            <div
              key={track.id}
              className="grid grid-cols-[1fr_auto_auto] gap-4 items-center py-5 px-4 hover:bg-white/5 rounded-2xl transition-colors group"
            >
              <div className="flex items-center gap-4">
                <div className="p-2 rounded-xl bg-[#B4A5FF]/10 group-hover:bg-[#B4A5FF]/20 transition-colors">
                  <svg
                    width="24"
                    height="24"
                    viewBox="0 0 24 24"
                    fill="none"
                    xmlns="http://www.w3.org/2000/svg"
                  >
                    <path
                      d="M12 3V13.55C11.41 13.21 10.73 13 10 13C7.79 13 6 14.79 6 17C6 19.21 7.79 21 10 21C12.21 21 14 19.21 14 17V7H18V3H12Z"
                      fill="#B4A5FF"
                    />
                  </svg>
                </div>
                <span className="font-medium">{track.name}</span>
              </div>
              <div className="flex items-center gap-3 text-gray-300">
                <svg
                  width="18"
                  height="18"
                  viewBox="0 0 24 24"
                  fill="none"
                  xmlns="http://www.w3.org/2000/svg"
                  className="opacity-50"
                >
                  <path
                    d="M11.99 2C6.47 2 2 6.48 2 12C2 17.52 6.47 22 11.99 22C17.52 22 22 17.52 22 12C22 6.48 17.52 2 11.99 2ZM12 20C7.58 20 4 16.42 4 12C4 7.58 7.58 4 12 4C16.42 4 20 7.58 20 12C20 16.42 16.42 20 12 20Z"
                    fill="currentColor"
                  />
                  <path
                    d="M12.5 7H11V13L16.25 16.15L17 14.92L12.5 12.25V7Z"
                    fill="currentColor"
                  />
                </svg>
                {track.duration}
              </div>
              <button 
                onClick={() => router.push(`/edit/${track.id}`)}
                className="bg-[#B4A5FF]/10 text-[#B4A5FF] px-5 py-2.5 rounded-xl hover:bg-[#B4A5FF]/20 transition-all hover:scale-105 font-medium"
              >
                Edit
              </button>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
