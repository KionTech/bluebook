import { loadPage } from "#lib/server/api.ts";
import type { OverviewProps } from "#lib/types.ts";

import type { PageServerLoad } from "./$types";

export const load: PageServerLoad = (event) => loadPage<OverviewProps>(event);
