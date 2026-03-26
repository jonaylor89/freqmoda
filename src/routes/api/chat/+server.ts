import { json } from '@sveltejs/kit';
import { env } from '$env/dynamic/private';
import { processAudio, getStreamingEngineUrl } from '$lib/server/streaming-engine';
import { findSample } from '$lib/samples';
import { resolvePresets } from '$lib/effect-presets';
import type { RequestHandler } from './$types';

const SYSTEM_PROMPT = `You are an audio processing assistant for FreqModa. You help users process and transform audio samples using various effects and transformations through natural language.

Available samples:
- Sample 1 (sample1.mp3): 32.9s
- Sample 2 (sample2.mp3): 32.9s
- Sample 3 (sample3.mp3): 32.9s
- Sample 4 (sample4.mp3): 32.9s
- Sample 5 (sample5.mp3): 130.6s
- Sample 6 (sample6.mp3): 32.9s
- Sample 7 (sample7.mp3): 32.9s
- Sample 8 (sample8.mp3): 32.9s

When the user asks you to process audio, respond with a JSON block describing the processing parameters. Use this exact format in your response:

\`\`\`json
{"audio_name": "sampleX.mp3", "reverse": true, "echo": "medium", ...}
\`\`\`

Available parameters:
- format: output format (mp3, wav, flac, ogg, m4a)
- start_time: start time in seconds
- duration: duration in seconds
- speed: playback speed multiplier
- reverse: true/false
- volume: volume multiplier
- normalize: true/false
- lowpass/highpass: filter cutoff in Hz
- bass/treble: boost/cut in dB
- fade_in/fade_out: duration in seconds
- echo: "light", "medium", or "heavy"
- chorus: "light", "medium", or "heavy"
- flanger: "light", "medium", or "heavy"

Always include the audio_name parameter. After the JSON block, provide a brief description of what you did.`;

export const POST: RequestHandler = async ({ request }) => {
	const { message, sampleKey, history } = await request.json();

	if (!message || typeof message !== 'string') {
		return json({ error: 'Message is required' }, { status: 400 });
	}

	if (!env.OPENAI_API_KEY) {
		return json({ error: 'OpenAI API key not configured' }, { status: 500 });
	}

	// Build message history for OpenAI
	const openaiMessages = [
		{ role: 'system' as const, content: SYSTEM_PROMPT },
		...(history || []).map((m: { role: string; content: string }) => ({
			role: m.role as 'user' | 'assistant',
			content: m.content,
		})),
	];

	// Add context about the current sample if working with one
	const enhancedMessage = sampleKey
		? `I'm working with the audio file "${sampleKey}". ${message}`
		: message;

	openaiMessages.push({ role: 'user' as const, content: enhancedMessage });

	try {
		// Call OpenAI
		const response = await fetch('https://api.openai.com/v1/chat/completions', {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json',
				Authorization: `Bearer ${env.OPENAI_API_KEY}`,
			},
			body: JSON.stringify({
				model: 'gpt-4o-mini',
				messages: openaiMessages,
				max_tokens: 1000,
			}),
		});

		if (!response.ok) {
			const errorText = await response.text();
			console.error('OpenAI error:', response.status, errorText);
			return json({ error: 'AI service error' }, { status: 502 });
		}

		const data = await response.json();
		const content = data.choices?.[0]?.message?.content || 'No response';

		// Try to extract JSON processing parameters from the response
		let audioUrl: string | undefined;
		let cleanContent = content;

		const jsonMatch = content.match(/```json\s*\n?([\s\S]*?)\n?```/);
		if (jsonMatch) {
			try {
				const params = JSON.parse(jsonMatch[1]);
				const audioName = params.audio_name || sampleKey || 'sample1.mp3';
				delete params.audio_name;

				// Resolve effect presets (e.g., "medium" -> actual values)
				const resolvedParams = resolvePresets(params);

				// Process audio through the streaming engine
				audioUrl = await processAudio(audioName, resolvedParams);

				// Clean the content - remove the JSON block
				cleanContent = content.replace(/```json\s*\n?[\s\S]*?\n?```/, '').trim();
				if (!cleanContent) {
					cleanContent = 'Audio processed successfully.';
				}
			} catch (err) {
				console.error('Failed to process audio params:', err);
				// Keep original content if processing fails
			}
		}

		return json({
			message: cleanContent,
			audioUrl,
		});
	} catch (err) {
		console.error('Chat error:', err);
		return json({ error: 'Internal server error' }, { status: 500 });
	}
};
