import { writable } from 'svelte/store';

type Theme = 'light' | 'dark';

function createThemeStore() {
  const stored = localStorage.getItem('theme') as Theme | null;
  const initial: Theme = stored || 'light';

  const { subscribe, set } = writable<Theme>(initial);

  return {
    subscribe,
    toggle: () => {
      set(initial === 'light' ? 'dark' : 'light');
    },
    set: (theme: Theme) => {
      localStorage.setItem('theme', theme);
      document.documentElement.setAttribute('data-theme', theme);
      set(theme);
    },
  };
}

export const themeStore = createThemeStore();
