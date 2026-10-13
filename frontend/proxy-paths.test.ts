import { describe, expect, it } from 'vitest';

import { isAxumRequest } from './proxy-paths.ts';

describe('isAxumRequest', () => {
  it('sends every mutation to the api', () => {
    for (const method of ['POST', 'PUT', 'PATCH', 'DELETE']) {
      expect(isAxumRequest(method, '/standings/1')).toBe(true);
      expect(isAxumRequest(method, '/dashboard')).toBe(true);
    }
  });

  it('sends the api paths to the api', () => {
    for (const path of [
      '/api/pages/home',
      '/api/health',
      '/up',
      '/eve',
      '/eve/callback',
      '/demo-login',
      '/storage/file.txt',
      '/settings',
    ]) {
      expect(isAxumRequest('GET', path), path).toBe(true);
    }
  });

  it('keeps the pages in SvelteKit', () => {
    for (const path of [
      '/',
      '/dashboard',
      '/settings/appearance',
      '/admin',
      '/admin/standing-requests',
      '/admin/pilots',
      '/admin/settings',
      '/favicon.svg',
    ]) {
      expect(isAxumRequest('GET', path), path).toBe(false);
    }
  });
});
