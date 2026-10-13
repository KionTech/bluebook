import { describe, expect, it, vi } from 'vitest';

import { Form, send, type FormEnvironment } from './form.svelte.ts';

function environment(status: number, body: unknown) {
  const calls: { url: string; init: RequestInit }[] = [];
  const env = {
    fetch: vi.fn(async (url: RequestInfo | URL, init?: RequestInit) => {
      calls.push({ url: String(url), init: init ?? {} });
      return new Response(JSON.stringify(body), { status });
    }) as unknown as typeof globalThis.fetch,
    goto: vi.fn(async () => {}),
    reload: vi.fn(async () => {}),
    leave: vi.fn(),
    error: vi.fn(),
  } satisfies FormEnvironment;
  return { env, calls };
}

describe('send', () => {
  it('sends JSON to the URL and stays when the api says so', async () => {
    const { env, calls } = environment(200, { redirect: null });

    const result = await send('PUT', '/standings/5', { should_sync: true }, env);

    expect(result).toEqual({ ok: true, errors: {} });
    expect(calls[0].url).toBe('/standings/5');
    expect(calls[0].init.method).toBe('PUT');
    expect(calls[0].init.body).toBe('{"should_sync":true}');
    expect(calls[0].init.headers).toEqual({
      'content-type': 'application/json',
      accept: 'application/json',
    });
    expect(env.reload).toHaveBeenCalledOnce();
    expect(env.goto).not.toHaveBeenCalled();
  });

  it('goes where the api sends it', async () => {
    const { env } = environment(200, { redirect: '/' });
    await send('POST', '/logout', {}, env);
    expect(env.goto).toHaveBeenCalledWith('/');
    expect(env.reload).not.toHaveBeenCalled();
  });

  it('returns the errors of a 422', async () => {
    const { env } = environment(422, { errors: { type: 'The selected type is invalid.' } });
    const result = await send('PUT', '/admin/standings', { type: 'x' }, env);
    expect(result).toEqual({ ok: false, errors: { type: 'The selected type is invalid.' } });
    expect(env.error).not.toHaveBeenCalled();
  });

  it('sends a guest to the login with a full page load', async () => {
    const { env } = environment(401, { message: 'Unauthenticated.', redirect: '/eve' });
    await send('POST', '/standings/sync', {}, env);
    expect(env.leave).toHaveBeenCalledWith('/eve');
  });

  it('shows the message of a refused form', async () => {
    const { env } = environment(403, { message: 'This action is unauthorized.' });
    const result = await send('PUT', '/standings/9', {}, env);
    expect(result.ok).toBe(false);
    expect(env.error).toHaveBeenCalledWith('This action is unauthorized.');
  });

  it('names the status when there is no message', async () => {
    const env = {
      ...environment(500, null).env,
      fetch: (async () => new Response('oops', { status: 500 })) as typeof globalThis.fetch,
    };
    await send('POST', '/setup/complete', {}, env);
    expect(env.error).toHaveBeenCalledWith('Server Error');
  });
});

describe('Form', () => {
  it('keeps the errors and the processing state', async () => {
    const { env } = environment(422, {
      errors: { webhook_url: 'The webhook url field must be a valid URL.' },
    });
    const form = new Form({ webhook_url: 'nope', role_id: '' }, env);

    const pending = form.submit('PUT', '/admin/discord-settings');
    expect(form.processing).toBe(true);
    expect(await pending).toBe(false);

    expect(form.processing).toBe(false);
    expect(form.errors).toEqual({ webhook_url: 'The webhook url field must be a valid URL.' });
  });
});
