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

// 使用独立 CSS 文件，避免组合版 @media prefers-color-scheme 的干扰
// github-markdown.css（组合版）会同时匹配系统和应用主题，导致冲突
// 分开加载确保颜色始终跟随 Zephyr 的主题设置
const MD_LIGHT_CSS = 'https://cdn.jsdelivr.net/npm/github-markdown-css@5.5.1/github-markdown-light.css';
const MD_DARK_CSS = 'https://cdn.jsdelivr.net/npm/github-markdown-css@5.5.1/github-markdown-dark.css';

export function initTheme() {
  const saved = localStorage.getItem(STORAGE_KEY);
  const isDark = saved === 'dark';
  // 始终调用 setDark 确保 CSS 主题文件与 data-theme 一致
  // 避免组合版 github-markdown.css 中 @media prefers-color-scheme 的干扰
  setDark(isDark);

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

  // 切换 markdown 内容区的样式表
  const mdLink = document.getElementById('md-theme-css');
  if (mdLink) {
    mdLink.href = dark ? MD_DARK_CSS : MD_LIGHT_CSS;
  }

  // 更新主题图标
  document.getElementById('theme-icon').outerHTML = dark ? SUN_SVG : MOON_SVG;
}
