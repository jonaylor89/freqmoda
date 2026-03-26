import { error } from '@sveltejs/kit';
import { samples } from '$lib/samples';
import { getStreamingEngineUrl } from '$lib/server/streaming-engine';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ params }) => {
  const sample = samples.find(s => s.streamingKey === params.key);
  if (!sample) {
    throw error(404, 'Sample not found');
  }

  return {
    sample,
    streamingEngineBaseUrl: getStreamingEngineUrl(),
  };
};
