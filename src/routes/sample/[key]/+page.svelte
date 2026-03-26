<script lang="ts">
  import { onMount } from 'svelte';
  import type { ChatMessage } from '$lib/types';

  let { data } = $props();

  let messages = $state<ChatMessage[]>([]);
  let inputValue = $state('');
  let isLoading = $state(false);
  let messagesEl: HTMLDivElement;
  let mainWaveform: HTMLDivElement;
  let wavesurfer: any = null;
  let isPlaying = $state(false);
  let currentTime = $state('00:00');
  let totalDuration = $state('00:00');

  const currentAudioUrl = $derived.by(() => {
    // Find the latest audio URL from messages, or use original sample
    for (let i = messages.length - 1; i >= 0; i--) {
      if (messages[i].audioUrl) return messages[i].audioUrl;
    }
    return `${data.streamingEngineBaseUrl}/unsafe/${data.sample.streamingKey}`;
  });

  function formatTime(seconds: number): string {
    const m = Math.floor(seconds / 60);
    const s = Math.floor(seconds % 60);
    return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
  }

  function scrollToBottom() {
    if (messagesEl) {
      messagesEl.scrollTop = messagesEl.scrollHeight;
    }
  }

  function fillExample(text: string) {
    inputValue = text;
  }

  async function initWaveform(url: string) {
    if (!mainWaveform) return;

    const WaveSurfer = (await import('wavesurfer.js')).default;

    if (wavesurfer) {
      wavesurfer.destroy();
    }

    wavesurfer = WaveSurfer.create({
      container: mainWaveform,
      waveColor: '#6366f1',
      progressColor: '#4f46e5',
      cursorColor: '#ef4444',
      barWidth: 3,
      barRadius: 2,
      height: 60,
      normalize: true,
    });

    wavesurfer.on('ready', () => {
      totalDuration = formatTime(wavesurfer.getDuration());
    });

    wavesurfer.on('timeupdate', (t: number) => {
      currentTime = formatTime(t);
    });

    wavesurfer.on('play', () => { isPlaying = true; });
    wavesurfer.on('pause', () => { isPlaying = false; });
    wavesurfer.on('finish', () => { isPlaying = false; });

    wavesurfer.load(url);
  }

  function togglePlay() {
    if (wavesurfer) {
      wavesurfer.playPause();
    }
  }

  async function handleSubmit() {
    const msg = inputValue.trim();
    if (!msg || isLoading) return;

    const userMessage: ChatMessage = {
      id: crypto.randomUUID(),
      role: 'user',
      content: msg,
    };
    messages = [...messages, userMessage];
    inputValue = '';
    isLoading = true;

    setTimeout(scrollToBottom, 0);

    try {
      const response = await fetch('/api/chat', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          message: msg,
          sampleKey: data.sample.streamingKey,
          history: messages.map(m => ({ role: m.role, content: m.content })),
        }),
      });

      if (!response.ok) throw new Error('Chat failed');

      const result = await response.json();

      const assistantMessage: ChatMessage = {
        id: crypto.randomUUID(),
        role: 'assistant',
        content: result.message,
        audioUrl: result.audioUrl,
      };
      messages = [...messages, assistantMessage];

      // If there's a new audio URL, update the main waveform
      if (result.audioUrl) {
        await initWaveform(result.audioUrl);
      }
    } catch (err) {
      const errorMessage: ChatMessage = {
        id: crypto.randomUUID(),
        role: 'assistant',
        content: 'Sorry, something went wrong. Please try again.',
      };
      messages = [...messages, errorMessage];
    } finally {
      isLoading = false;
      setTimeout(scrollToBottom, 0);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSubmit();
    }
  }

  onMount(() => {
    initWaveform(`${data.streamingEngineBaseUrl}/unsafe/${data.sample.streamingKey}`);
  });
</script>

<div class="h-screen flex flex-col">
  <!-- Header -->
  <header class="bg-white border-b border-gray-200 px-4 sm:px-6 py-3">
    <div class="flex items-center justify-between">
      <div class="flex items-center space-x-3">
        <a
          href="/"
          class="flex items-center space-x-2 px-3 py-2 rounded-lg border border-gray-200 hover:bg-gray-100 text-sm font-medium text-gray-700"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
          </svg>
          <span>Back</span>
        </a>
        <div class="hidden sm:block w-px h-6 bg-gray-300"></div>
        <h1 class="text-lg font-bold text-gray-900 truncate">{data.sample.title}</h1>
      </div>
    </div>
  </header>

  <!-- Waveform -->
  <div class="bg-white border-b border-gray-200 px-4 sm:px-6 py-4">
    <div class="flex items-center space-x-4">
      <button
        onclick={togglePlay}
        class="bg-green-600 hover:bg-green-700 text-white w-10 h-10 rounded-full flex items-center justify-center flex-shrink-0"
      >
        {#if isPlaying}
          <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
            <rect x="5" y="3" width="4" height="14" />
            <rect x="11" y="3" width="4" height="14" />
          </svg>
        {:else}
          <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
            <path d="M6.3 2.84A1.5 1.5 0 004 4.11v11.78a1.5 1.5 0 002.3 1.27l9.344-5.891a1.5 1.5 0 000-2.538L6.3 2.84z" />
          </svg>
        {/if}
      </button>
      <div class="flex-1 min-w-0">
        <div class="text-sm text-gray-500 font-mono mb-1">{currentTime} / {totalDuration}</div>
        <div bind:this={mainWaveform} class="rounded-lg border border-gray-200 bg-gray-50 min-h-[60px]"></div>
      </div>
    </div>
  </div>

  <!-- Chat Messages -->
  <div class="flex-1 flex flex-col min-h-0">
    <div class="flex-1 overflow-hidden">
      <div
        bind:this={messagesEl}
        class="h-full overflow-y-auto px-4 sm:px-6 py-6 space-y-4"
        style="scrollbar-width: thin;"
      >
        {#if messages.length === 0}
          <div class="flex items-center justify-center h-full">
            <div class="max-w-md text-center space-y-4">
              <h3 class="text-xl font-bold text-gray-900">Ready to transform {data.sample.title}</h3>
              <p class="text-gray-500">Describe how you want to process this audio sample.</p>
              <div class="space-y-2 text-left">
                <p class="text-sm font-medium text-gray-600 mb-2">Try these examples:</p>
                <button onclick={() => fillExample('Reverse and add echo')} class="w-full p-3 bg-white border border-gray-200 rounded-lg text-left hover:bg-gray-50 text-sm">
                  "Reverse and add echo"
                </button>
                <button onclick={() => fillExample('Make it play faster with a fade in')} class="w-full p-3 bg-white border border-gray-200 rounded-lg text-left hover:bg-gray-50 text-sm">
                  "Make it play faster with a fade in"
                </button>
                <button onclick={() => fillExample('Add chorus effect')} class="w-full p-3 bg-white border border-gray-200 rounded-lg text-left hover:bg-gray-50 text-sm">
                  "Add chorus effect"
                </button>
              </div>
            </div>
          </div>
        {:else}
          {#each messages as message (message.id)}
            <div class="flex space-x-3 {message.role === 'user' ? 'flex-row-reverse space-x-reverse' : ''}">
              <div class="flex-shrink-0">
                <div class="w-8 h-8 rounded-full flex items-center justify-center text-white text-xs font-bold {message.role === 'user' ? 'bg-indigo-500' : 'bg-green-600'}">
                  {message.role === 'user' ? 'U' : 'AI'}
                </div>
              </div>
              <div class="flex-1 max-w-[85%]">
                <div class="{message.role === 'user' ? 'bg-indigo-500 text-white ml-auto' : 'bg-white border border-gray-200 text-gray-800'} rounded-2xl px-4 py-3">
                  <div class="text-sm whitespace-pre-wrap">{message.content}</div>
                  {#if message.audioUrl}
                    <div class="mt-2 pt-2 border-t {message.role === 'user' ? 'border-indigo-400' : 'border-gray-100'}">
                      <audio controls class="w-full h-8" src={message.audioUrl}>
                        <track kind="captions" />
                      </audio>
                    </div>
                  {/if}
                </div>
              </div>
            </div>
          {/each}

          {#if isLoading}
            <div class="flex space-x-3">
              <div class="flex-shrink-0">
                <div class="w-8 h-8 rounded-full flex items-center justify-center bg-green-600 text-white text-xs font-bold">AI</div>
              </div>
              <div class="bg-white border border-gray-200 rounded-2xl px-4 py-3">
                <div class="flex items-center space-x-2 text-gray-500">
                  <span class="text-sm">Thinking</span>
                  <div class="flex space-x-1">
                    <div class="w-1.5 h-1.5 bg-gray-400 rounded-full animate-bounce" style="animation-delay: -0.32s"></div>
                    <div class="w-1.5 h-1.5 bg-gray-400 rounded-full animate-bounce" style="animation-delay: -0.16s"></div>
                    <div class="w-1.5 h-1.5 bg-gray-400 rounded-full animate-bounce"></div>
                  </div>
                </div>
              </div>
            </div>
          {/if}
        {/if}
      </div>
    </div>

    <!-- Chat Input -->
    <div class="border-t border-gray-200 bg-white p-4">
      <form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
        <div class="flex space-x-3">
          <input
            type="text"
            bind:value={inputValue}
            onkeydown={handleKeydown}
            maxlength={2000}
            class="flex-1 px-4 py-2.5 border border-gray-300 rounded-xl focus:ring-2 focus:ring-indigo-500 focus:border-transparent outline-none text-sm"
            placeholder="Describe how to process {data.sample.title}..."
            disabled={isLoading}
          />
          <button
            type="submit"
            class="bg-indigo-500 hover:bg-indigo-600 text-white px-5 py-2.5 rounded-xl text-sm font-medium disabled:opacity-50"
            disabled={isLoading}
          >
            {isLoading ? 'Sending...' : 'Send'}
          </button>
        </div>
      </form>
    </div>
  </div>
</div>
