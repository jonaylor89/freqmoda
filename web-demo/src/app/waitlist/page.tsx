'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';

export default function Waitlist() {
  const router = useRouter();
  const [email, setEmail] = useState('');
  const [agreed, setAgreed] = useState(false);

  return (
    <div className="min-h-screen bg-gradient-to-br from-[#13111C] to-[#191627] text-white p-8">
      <div className="max-w-xl mx-auto">
        <button 
          onClick={() => router.back()}
          className="text-[#B4A5FF] hover:text-white transition-colors mb-12 flex items-center gap-2"
        >
          <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
            <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"/>
          </svg>
          Back
        </button>

        <div className="bg-[#1C1A27]/70 backdrop-blur-xl rounded-3xl p-8 shadow-2xl border border-white/5">
          <h1 className="text-4xl font-bold bg-gradient-to-r from-[#B4A5FF] to-[#7C66FF] bg-clip-text text-transparent mb-6">
            Join the Waiting List
          </h1>
          
          <div className="space-y-6">
            <div>
              <label className="block text-gray-300 mb-2 text-sm">Email address</label>
              <input
                type="email"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                className="w-full bg-[#13111C] border border-white/10 rounded-xl px-4 py-3 focus:outline-none focus:border-[#B4A5FF]/50 transition-colors"
                placeholder="Enter your email"
              />
            </div>

            <div className="flex items-start gap-3">
              <input
                type="checkbox"
                id="consent"
                checked={agreed}
                onChange={(e) => setAgreed(e.target.checked)}
                className="mt-1 accent-[#B4A5FF]"
              />
              <label htmlFor="consent" className="text-sm text-gray-300 leading-relaxed">
                I agree to receive communications about the product. You can unsubscribe at any time.
              </label>
            </div>

            <button
              disabled={!email || !agreed}
              className="w-full bg-gradient-to-r from-[#B4A5FF] to-[#7C66FF] py-3 rounded-xl font-medium disabled:opacity-50 disabled:cursor-not-allowed hover:from-[#9785FF] hover:to-[#6A54FF] transition-all"
            >
              Join Waitlist
            </button>
          </div>
        </div>
      </div>
    </div>
  );
} 