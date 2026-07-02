import state from '../state.js';

const STORAGE_KEY = 'zephyr-theme';

export function initTheme() {
  const saved = localStorage.getItem(STORAGE_KEY);
  if (saved === 'dark') {
    setDark(true);
  }

  document.getElementById('toggle-theme').addEventListener('click', toggleTheme);
}

function toggleTheme() {
  const isDark = !state.darkMode;
  setDark(isDark);
  localStorage.setItem(STORAGE_KEY, isDark ? 'dark' : 'light');
}

function setDark(dark) {
  state.darkMode = dark;
  document.documentElement.setAttribute('data-theme', dark ? 'dark' : 'light');
  document.getElementById('toggle-theme').textContent = dark ? '☀️' : '🌙';
}
