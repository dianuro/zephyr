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

// github-markdown-css 文件映射
const MD_LIGHT_CSS = 'https://cdn.jsdelivr.net/npm/github-markdown-css@5.5.1/github-markdown-light.css';
const MD_DARK_CSS = 'https://cdn.jsdelivr.net/npm/github-markdown-css@5.5.1/github-markdown-dark.css';

export async function initTheme() {
  const saved = localStorage.getItem(STORAGE_KEY);
  const isDark = saved === 'dark';
  await setDark(isDark);

  document.getElementById('toggle-theme').addEventListener('click', toggleTheme);
}

async function toggleTheme() {
  const isDark = !state.darkMode;
  await setDark(isDark);
  localStorage.setItem(STORAGE_KEY, isDark ? 'dark' : 'light');
}

async function setDark(dark) {
  state.darkMode = dark;
  document.documentElement.setAttribute('data-theme', dark ? 'dark' : 'light');

  // 1. 切换 markdown 内容区的样式表
  const mdLink = document.getElementById('md-theme-css');
  if (mdLink) {
    mdLink.href = dark ? MD_DARK_CSS : MD_LIGHT_CSS;
  }

  // 2. 从 Rust 后端加载主题配置，应用到 CSS 变量
  try {
    const { invoke } = window.__TAURI__.core;
    const config = await invoke('get_theme_config', { isDark: dark });

    // 将配置映射为 CSS 自定义属性
    applyConfig(config);
  } catch (e) {
    console.warn('无法加载主题配置:', e);
  }

  // 3. 更新主题图标
  const themeIcon = document.getElementById('theme-icon');
  if (themeIcon) {
    themeIcon.outerHTML = dark ? SUN_SVG : MOON_SVG;
  }
}

function applyConfig(cfg) {
  const root = document.documentElement;

  // app
  root.style.setProperty('--bg-primary', cfg.app.background);
  root.style.setProperty('--bg-secondary', cfg.app.sidebar_bg);
  root.style.setProperty('--bg-sidebar', cfg.app.sidebar_bg);
  root.style.setProperty('--bg-toolbar', cfg.app.toolbar_bg);
  root.style.setProperty('--bg-content', cfg.app.content_bg);
  root.style.setProperty('--bg-hover', cfg.app.hover_bg);

  // text
  root.style.setProperty('--text-primary', cfg.text.primary);
  root.style.setProperty('--text-secondary', cfg.text.secondary);
  root.style.setProperty('--text-muted', cfg.text.muted);

  // border
  root.style.setProperty('--border-color', cfg.border.default);
  root.style.setProperty('--border-muted', cfg.border.muted);

  // accent
  root.style.setProperty('--accent', cfg.accent.default);
  root.style.setProperty('--accent-hover', cfg.accent.hover);

  // scrollbar
  root.style.setProperty('--scrollbar-thumb', cfg.scrollbar.thumb);
  root.style.setProperty('--scrollbar-thumb-hover', cfg.scrollbar.thumb_hover);

  // search
  root.style.setProperty('--search-highlight-bg', cfg.search.highlight_bg);
  root.style.setProperty('--search-active-bg', cfg.search.active_bg);
}
