import { loadPage } from "#lib/server/api.ts";
import type { HomeProps } from "#lib/types.ts";

import type { PageServerLoad } from "./$types";

export const load: PageServerLoad = (event) => loadPage<HomeProps>(event);
