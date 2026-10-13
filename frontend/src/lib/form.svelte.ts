// Forms as the api answers them: the fields go as JSON to the URL and
// method the site always used. `200 {"redirect"}` either stays (reload the
// page's data, Laravel's `back()`) or goes to a path; `422 {"errors"}`
// holds each field's message; a guest is sent to the login (`401` with
// its URL, a full page load); anything else becomes an error toast. The
// flash message arrives with the reloaded page.
import { goto, refreshAll } from '$app/navigation';

import { errorTitle } from '#lib/errors.ts';
import { toaster } from '#lib/toaster.svelte.ts';

export type FormMethod = 'POST' | 'PUT' | 'PATCH' | 'DELETE';

/** What a form needs from SvelteKit, the browser and the toaster; tests
 * pass fakes. */
export type FormEnvironment = {
  fetch: typeof globalThis.fetch;
  goto: (url: string) => Promise<void>;
  reload: () => Promise<void>;
  leave: (url: string) => void;
  error: (message: string) => void;
};

export type SendResult = { ok: boolean; errors: Record<string, string> };

type Body = { redirect?: string | null; errors?: Record<string, string>; message?: string };

function defaults(environment: Partial<FormEnvironment>): FormEnvironment {
  return {
    fetch: (input, init) => globalThis.fetch(input, init),
    goto: (url) => goto(url, { refreshAll: true }),
    reload: () => refreshAll(),
    leave: (url) => {
      window.location.href = url;
    },
    error: (message) => toaster.add('error', message),
    ...environment,
  };
}

/** Sends a form and follows the answer. */
export async function send(
  method: FormMethod,
  url: string,
  payload: Record<string, unknown> = {},
  environment: Partial<FormEnvironment> = {},
): Promise<SendResult> {
  const env = defaults(environment);
  let response: Response;
  try {
    response = await env.fetch(url, {
      method,
      headers: { 'content-type': 'application/json', accept: 'application/json' },
      body: JSON.stringify(payload),
    });
  } catch {
    env.error(errorTitle(500));
    return { ok: false, errors: {} };
  }
  const body: Body = await response.json().catch(() => ({}));
  if (response.ok) {
    if (body.redirect) {
      await env.goto(body.redirect);
    } else {
      await env.reload();
    }
    return { ok: true, errors: {} };
  }
  if (response.status === 422) {
    return { ok: false, errors: body.errors ?? {} };
  }
  if (response.status === 401) {
    env.leave(body.redirect ?? '/eve');
  } else {
    env.error(body.message ?? errorTitle(response.status));
  }
  return { ok: false, errors: {} };
}

/** A form's fields and state, Inertia's useForm for this protocol. */
export class Form<T extends Record<string, unknown>> {
  data: T;
  errors: Record<string, string> = $state({});
  processing = $state(false);
  #environment: Partial<FormEnvironment>;

  constructor(initial: T, environment: Partial<FormEnvironment> = {}) {
    this.data = $state(initial);
    this.#environment = environment;
  }

  /** Sends the fields; resolves to whether the form went through. */
  async submit(method: FormMethod, url: string): Promise<boolean> {
    this.processing = true;
    try {
      const result = await send(
        method,
        url,
        $state.snapshot(this.data) as Record<string, unknown>,
        this.#environment,
      );
      this.errors = result.errors;
      return result.ok;
    } finally {
      this.processing = false;
    }
  }
}
