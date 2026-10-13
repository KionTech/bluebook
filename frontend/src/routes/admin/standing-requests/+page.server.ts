import { loadPage } from '#lib/server/api.ts';
import type { StandingRequestsProps } from '#lib/types.ts';

import type { PageServerLoad } from './$types';

export const load: PageServerLoad = (event) => loadPage<StandingRequestsProps>(event);
