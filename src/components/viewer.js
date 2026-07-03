import state from '../state.js';
import { renderOutline } from './outline.js';
import { buildFileTree } from './sidebar.js';

export async function openFile(path) {
  try {
    const { invoke } = window.__TAURI__.core;
    const isDark = document.documentElement.getAttribute('data-theme') === 'dark';
    const result = await invoke('open_file', { path, isDark });

    state.currentFile = path;
    state.currentHtml = result.html;
    state.metadata = result.metadata;

    // 更新标题
    const fileName = path.split('/').pop() || path.split('\\').pop();
    const title = result.metadata.title || fileName;
    document.getElementById('file-title').textContent = title;
    document.title = `${title} — Zephyr`;

    // 隐藏欢迎页，显示内容
    document.getElementById('welcome').classList.add('hidden');
    const contentEl = document.getElementById('content');
    contentEl.classList.remove('hidden');
    contentEl.innerHTML = result.html;

    // 后处理：KaTeX + Mermaid
    await renderMath();
    renderDiagrams();

    // 更新侧边栏
    renderOutline(result.metadata.headings);
    buildFileTree(path);

    // 自动显示侧边栏（如果隐藏）
    if (!state.sidebarVisible) {
      document.getElementById('toggle-sidebar').click();
    }

  } catch (e) {
    console.error('打开文件失败:', e);
    showError(`无法打开文件: ${e}`);
  }
}

async function renderMath() {
  // KaTeX 使用 `language-math` 代码块
  const mathBlocks = document.querySelectorAll('#content code.language-math');
  if (mathBlocks.length === 0) return;

  try {
    const katex = await import('https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.mjs');
    mathBlocks.forEach(block => {
      const text = block.textContent;
      const isDisplay = block.closest('pre') !== null;
      try {
        const html = katex.renderToString(text, {
          displayMode: isDisplay,
          throwOnError: false,
        });
        if (isDisplay) {
          const pre = block.closest('pre');
          if (pre) pre.outerHTML = html;
        } else {
          block.outerHTML = html;
        }
      } catch (err) {
        // 渲染失败，保留原始代码
        console.warn('KaTeX 渲染失败:', err);
      }
    });
  } catch (e) {
    console.warn('KaTeX 加载失败:', e);
  }
}

async function renderDiagrams() {
  const mermaidBlocks = document.querySelectorAll('#content code.language-mermaid');
  if (mermaidBlocks.length === 0) return;

  try {
    const mermaid = await import('https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.esm.min.mjs');
    mermaid.default.initialize({
      startOnLoad: false,
      theme: state.darkMode ? 'dark' : 'default',
    });

    mermaidBlocks.forEach((block, index) => {
      const text = block.textContent;
      const pre = block.closest('pre');
      if (!pre) return;

      const id = `zephyr-mermaid-${index}`;
      mermaid.default.render(id, text).then(({ svg }) => {
        pre.outerHTML = svg;
      }).catch(err => {
        console.warn('Mermaid 渲染失败:', err);
      });
    });
  } catch (e) {
    console.warn('Mermaid 加载失败:', e);
  }
}

function showError(msg) {
  document.getElementById('welcome').classList.add('hidden');
  const contentEl = document.getElementById('content');
  contentEl.classList.remove('hidden');
  contentEl.innerHTML = `<div class="error-message">
    <div class="error-icon">
      <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="#cf222e" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="10"/>
        <line x1="15" y1="9" x2="9" y2="15"/>
        <line x1="9" y1="9" x2="15" y2="15"/>
      </svg>
    </div>
    <h2>出错了</h2>
    <p>${escapeHtml(msg)}</p></div>`;
}

function escapeHtml(str) {
  const div = document.createElement('div');
  div.textContent = str;
  return div.innerHTML;
}

// ====== 外部链接处理 ======

/** 初始化外部链接拦截（事件委托，只绑一次） */
export function initExternalLinks() {
  const content = document.getElementById('content');
  if (!content) return;

  content.addEventListener('click', (e) => {
    // 向上查找 <a> 标签
    let target = e.target;
    while (target && target !== content) {
      if (target.tagName === 'A' && target.href) {
        const href = target.getAttribute('href');
        // 只处理 http/https 外部链接
        if (href && (href.startsWith('http://') || href.startsWith('https://'))) {
          e.preventDefault();
          e.stopPropagation();
          openInBrowser(href);
          return;
        }
        // 其他链接（#锚点、相对路径等）让默认行为处理
        break;
      }
      target = target.parentElement;
    }
  });
}

async function openInBrowser(url) {
  try {
    const { invoke } = window.__TAURI__.core;
    await invoke('plugin:opener|open_url', { url });
  } catch (e) {
    console.warn('打开外部链接失败:', e);
    // 回退：使用 window.open
    window.open(url, '_blank');
  }
}
