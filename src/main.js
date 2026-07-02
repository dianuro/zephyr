import state from './state.js';
import { initSidebar, toggleSidebar } from './components/sidebar.js';
import { openFile } from './components/viewer.js';
import { initSearch } from './components/search.js';
import { initTheme } from './components/theme.js';

async function init() {
  initSidebar();
  initSearch();
  initTheme();

  setupNativeDragDrop();
  setupFileWatcher();
  handleCliArgs();

  // 快捷键：Ctrl+\ 切换侧边栏
  document.addEventListener('keydown', (e) => {
    if ((e.ctrlKey || e.metaKey) && e.key === '\\') {
      e.preventDefault();
      toggleSidebar();
    }
  });

  console.log('Zephyr 已启动');
}

function setupNativeDragDrop() {
  // 使用 Tauri 2 原生拖拽事件（能得到文件路径）
  if (window.__TAURI__) {
    const { listen } = window.__TAURI__.event;
    listen('tauri://drag-drop', async (event) => {
      const payload = event.payload;
      // Tauri 2 drag-drop payload: { type: 'over' | 'drop', paths: string[] }
      if (payload && payload.type === 'drop' && payload.paths) {
        for (const path of payload.paths) {
          if (path.endsWith('.md') || path.endsWith('.markdown')) {
            await openFile(path);
            return;
          }
        }
      }
    });
  }

  // 额外保留 HTML5 拖拽作为备选
  setupHtmlDragDrop();
}

function setupHtmlDragDrop() {
  document.addEventListener('dragover', (e) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'copy';
  });

  document.addEventListener('drop', async (e) => {
    e.preventDefault();
    const files = Array.from(e.dataTransfer.files);
    const mdFile = files.find(f => f.name.endsWith('.md') || f.name.endsWith('.markdown'));

    if (!mdFile) return;

    // 在 Tauri 中，拖拽的 File 可能有 path 属性（某些 WebKit 版本）
    const path = mdFile.path || mdFile.webkitRelativePath;
    if (path) {
      await openFile(path);
    } else {
      // 纯浏览器环境：通过 FileReader 读取并调用 Rust 渲染
      const reader = new FileReader();
      reader.onload = async (evt) => {
        const content = evt.target?.result;
        if (typeof content === 'string') {
          try {
            const { invoke } = window.__TAURI__.core;
            const result = await invoke('render_markdown', { content });
            state.currentFile = mdFile.name;
            state.currentHtml = result.html;
            state.metadata = result.metadata;
            document.getElementById('file-title').textContent = mdFile.name;
            document.title = `${mdFile.name} — Zephyr`;
            document.getElementById('welcome').classList.add('hidden');
            const contentEl = document.getElementById('content');
            contentEl.classList.remove('hidden');
            contentEl.innerHTML = result.html;
          } catch (err) {
            console.error('渲染失败:', err);
          }
        }
      };
      reader.readAsText(mdFile);
    }
  });
}

function setupFileWatcher() {
  if (window.__TAURI__) {
    const { listen } = window.__TAURI__.event;
    listen('file-changed', async (event) => {
      const path = event.payload;
      if (path === state.currentFile) {
        console.log('文件已修改，自动重新加载:', path);
        await openFile(path);
      }
    });
  }
}

function handleCliArgs() {
  // 等待页面加载，然后检查 CLI 参数
  setTimeout(async () => {
    if (window.__ZEPHYR_CLI_FILE__) {
      const path = window.__ZEPHYR_CLI_FILE__;
      if (path) {
        await openFile(path);
      }
    }
  }, 500);
}

if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', init);
} else {
  init();
}
