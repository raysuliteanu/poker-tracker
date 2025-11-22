import { describe, it, expect, beforeEach, vi } from 'vitest';
import { get } from 'svelte/store';

describe('themeStore', () => {
  beforeEach(async () => {
    vi.resetModules();
    localStorage.clear();
    document.documentElement.removeAttribute('data-theme');
  });

  it('initializes with light theme when no theme in localStorage', async () => {
    const { themeStore } = await import('../../stores/theme');
    const theme = get(themeStore);

    expect(theme).toBe('light');
  });

  it('initializes with stored theme from localStorage', async () => {
    localStorage.setItem('theme', 'dark');

    const { themeStore } = await import('../../stores/theme');
    const theme = get(themeStore);

    expect(theme).toBe('dark');
  });

  it('set() updates theme, localStorage, and document attribute', async () => {
    const { themeStore } = await import('../../stores/theme');

    themeStore.set('dark');

    expect(get(themeStore)).toBe('dark');
    expect(localStorage.getItem('theme')).toBe('dark');
    expect(document.documentElement.getAttribute('data-theme')).toBe('dark');
  });

  it('set() can switch back to light theme', async () => {
    localStorage.setItem('theme', 'dark');

    const { themeStore } = await import('../../stores/theme');
    themeStore.set('light');

    expect(get(themeStore)).toBe('light');
    expect(localStorage.getItem('theme')).toBe('light');
    expect(document.documentElement.getAttribute('data-theme')).toBe('light');
  });

  it('toggle() switches from light to dark when initialized as light', async () => {
    const { themeStore } = await import('../../stores/theme');

    themeStore.toggle();

    expect(get(themeStore)).toBe('dark');
  });

  it('toggle() switches from dark to light when initialized as dark', async () => {
    localStorage.setItem('theme', 'dark');

    const { themeStore } = await import('../../stores/theme');

    themeStore.toggle();

    expect(get(themeStore)).toBe('light');
  });
});
