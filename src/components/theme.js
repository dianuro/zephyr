import state from '../state.js';

const STORAGE_KEY = 'zephyr-theme';

// Sun icon SVG (for dark→light toggle)
const SUN_SVG = `<svg id="theme-icon" width="18" height="18" viewBox="0 0 18 18" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
  <circle cx="9" cy="9" r="3.5"/>
  <line x1="9" y1="1.5" x2="9" y2="3"/>
  <line x1="9" y1="15" x2="9" y2="16.5"/>
  <line x1="1.5" y1="9" x2="3" y2="9"/>
  <line x1="15" y1="9" x2="16.5" y2="9"/>
  <line x1="3.8" y1="3.8" x2="4.9" y2="4.9"/>
  <line x1="13.1" y1="13.1" x2="14.2" y2="14.2"/>
  <line x1="3.8" y1="14.2" x2="4.9" y2="13.1"/>
  <line x1="13.1" y1="4.9" x2="14.2" y2="3.8"/>
</svg>`;

// Moon icon SVG (for light→dark toggle)
const MOON_SVG = `<svg id="theme-icon" width="18" height="18" viewBox="0 0 18 18" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
  <path d="M14.5 11.5A6 6 0 0 1 6.5 3.5a6 6 0 1 0 8 8z"/>
</svg>`;

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
  document.getElementById('theme-icon').outerHTML = dark ? SUN_SVG : MOON_SVG;
}
