import { describe, expect, it } from 'vitest';

import { appearanceOf, applyAppearance } from './appearance.ts';

describe('appearanceOf', () => {
  it('reads the cookie and falls back to dark', () => {
    expect(appearanceOf('dark')).toBe('dark');
    expect(appearanceOf('light')).toBe('light');
    expect(appearanceOf('system')).toBe('system');
    expect(appearanceOf(undefined)).toBe('dark');
    expect(appearanceOf("x';alert(1)//")).toBe('dark');
  });
});

describe('applyAppearance', () => {
  it('marks the dark theme on the root element', () => {
    const html = '<html class="%appearance.class%"><script>"%appearance.value%"</script>';
    expect(applyAppearance(html, 'dark')).toBe('<html class="dark"><script>"dark"</script>');
    expect(applyAppearance(html, 'system')).toBe('<html class=""><script>"system"</script>');
  });
});
