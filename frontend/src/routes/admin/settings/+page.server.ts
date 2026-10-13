import { loadPage } from '#lib/server/api.ts';

import type { PageServerLoad } from './$types';

export const load: PageServerLoad = (event) => loadPage(event);
