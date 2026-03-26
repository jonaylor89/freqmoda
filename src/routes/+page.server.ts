import { samples } from '$lib/samples';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async () => {
  return { samples };
};
