import { json } from '@sveltejs/kit';
import { healthCheck } from '$lib/server/streaming-engine';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async () => {
	const streamingEngineHealthy = await healthCheck();

	return json({
		status: 'healthy',
		services: {
			streaming_engine: streamingEngineHealthy ? 'healthy' : 'unhealthy',
		},
	});
};
