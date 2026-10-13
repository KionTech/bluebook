import { describe, expect, it } from 'vitest';

import { linkLabel, sitePath } from './pagination.ts';

describe('pagination', () => {
  it('decodes the arrows of the labels', () => {
    expect(linkLabel('&laquo; Previous')).toBe('« Previous');
    expect(linkLabel('Next &raquo;')).toBe('Next »');
    expect(linkLabel('2')).toBe('2');
  });

  it('turns the paginator URLs into site paths', () => {
    expect(
      sitePath(
        'https://bluebook.nicolaskion.dev/admin/pilots?view=corporations&search=Corp&page=2',
      ),
    ).toBe('/admin/pilots?view=corporations&search=Corp&page=2');
  });
});
